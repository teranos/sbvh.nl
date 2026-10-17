use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use qntx_grpc::plugin::proto::ats_store_service_client::AtsStoreServiceClient;
use qntx_grpc::plugin::proto::domain_plugin_service_server::DomainPluginService;
use qntx_grpc::plugin::proto::mail_service_client::MailServiceClient;
use qntx_grpc::plugin::proto::{
    attestation_filter, service_endpoint, Attestation, AttestationFilter, ConfigFieldSchema,
    ConfigSchemaResponse, ElementDefResponse, Empty, ExecuteJobRequest, ExecuteJobResponse,
    GenerateAttestationRequest, GetAttestationsRequest, HealthResponse, HttpRequest, HttpResponse,
    InitializeRequest, InitializeResponse, MetadataResponse, RouteInfo, SendMailRequest, ServiceEndpoint,
    SetMailTemplateRequest, ShutdownResponse, SigilsFrom, WebSocketMessage,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::{error, info};

use crate::meet::{self, Calendar, Held};

/// The last segment of the plugin's repository URL is its name (ADR-002:
/// https://github.com/teranos/QNTX/blob/main/docs/adr/ADR-002-plugin-configuration.md).
/// The loader refuses to load on mismatch.
pub(crate) const PLUGIN: &str = "sbvhAPI";

/// How many rows a read that wants every row asks the node for. A query names
/// how many rows it wants; none named is none given.
const EVERY_ROW: i32 = i32::MAX;

/// The keys Initialize reads off the plugin's record, besides QNTX's own.
const OWNER_KEY: &str = "owner";
const PLACE_KEY: &str = "place";

#[derive(Default)]
struct State {
    initialized: bool,
    ats_endpoint: String,
    mail_endpoint: Option<ServiceEndpoint>,
    auth_token: String,
    /// The calendar the record describes, or why it describes none. Held as
    /// the refusal rather than replaced by the defaults, so a record with a
    /// typo in its hours says so instead of offering hours nobody chose.
    calendar: Option<Result<Calendar, String>>,
    /// The User told of every call. None is a record naming nobody.
    owner: Option<String>,
    /// Where the call happens, as both mails say it.
    place: String,
    sentry: Option<sentry::ClientInitGuard>,
}

/// The address a service the node runs for plugins answers at, or why this
/// plugin cannot reach it, the node's own reason when it gave one.
fn reach(initialized: bool, endpoint: &Option<ServiceEndpoint>, service: &str) -> Result<String, String> {
    if !initialized {
        return Err(format!("no {service} endpoint; not initialized"));
    }
    match endpoint.as_ref().map(|e| e.state.as_ref()) {
        Some(Some(service_endpoint::State::Address(address))) => Ok(address.clone()),
        Some(Some(service_endpoint::State::NotRunning(why))) => {
            Err(format!("this node's QNTX runs without {service}: {why}"))
        }
        Some(None) => Err(format!(
            "the node's {service} endpoint names neither an address nor why it is not running"
        )),
        None => Err(format!("the node handed no {service} endpoint")),
    }
}

/// One call as the store holds it.
#[derive(Debug, Clone)]
struct Booked {
    meet_id: String,
    user: String,
    identity: String,
    start: i64,
    minutes: i64,
    about: String,
    name: String,
    email: String,
}

impl Booked {
    fn of(a: &Attestation) -> Option<Booked> {
        let attributes = crate::attest::attributes_of(a);
        let text = |k: &str| attributes.get(k).and_then(|v| v.as_str()).unwrap_or_default().to_string();
        Some(Booked {
            meet_id: a.subjects.first()?.clone(),
            user: a.actors.first()?.clone(),
            identity: text("identity"),
            start: crate::attest::whole(attributes.get("start"))?,
            minutes: crate::attest::whole(attributes.get("minutes"))?,
            about: text("about"),
            name: text("name"),
            email: text("email"),
        })
    }

    fn ends(&self) -> i64 {
        self.start + self.minutes * 60
    }

    fn held(&self) -> Held {
        Held { start: self.start, minutes: self.minutes }
    }

    fn call<'a>(&'a self, place: &'a str) -> crate::mail::Call<'a> {
        crate::mail::Call {
            start: self.start,
            minutes: self.minutes,
            about: &self.about,
            name: &self.name,
            email: &self.email,
            place,
        }
    }

    /// What a prospect is shown of their own call.
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "meet_id": self.meet_id,
            "start":   self.start,
            "minutes": self.minutes,
            "about":   self.about,
        })
    }

    /// What ROOT is shown: the call and who booked it.
    fn to_json_whole(&self) -> serde_json::Value {
        serde_json::json!({
            "meet_id":  self.meet_id,
            "start":    self.start,
            "minutes":  self.minutes,
            "about":    self.about,
            "name":     self.name,
            "email":    self.email,
            "user":     self.user,
            "identity": self.identity,
        })
    }
}

/// Every call that has not ended and was not cancelled, earliest first.
fn standing(booked: &[Attestation], cancelled: &[Attestation], now: i64) -> Vec<Booked> {
    let gone: HashSet<&str> = cancelled
        .iter()
        .filter_map(|a| a.subjects.first().map(String::as_str))
        .collect();
    let mut calls: Vec<Booked> = booked
        .iter()
        .filter_map(Booked::of)
        .filter(|b| !gone.contains(b.meet_id.as_str()) && b.ends() > now)
        .collect();
    calls.sort_by_key(|b| b.start);
    calls
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}

/// One header core sets on every request it hands a plugin. Nothing a caller
/// sends survives as one of these (proxyHTTPRequest).
fn header(req: &HttpRequest, name: &str) -> Option<String> {
    req.headers
        .iter()
        .find(|h| h.name.eq_ignore_ascii_case(name))
        .and_then(|h| h.values.first())
        .filter(|v| !v.is_empty())
        .cloned()
}

/// The User core says is asking. Empty is nobody.
fn asker(req: &HttpRequest) -> Option<String> {
    header(req, "X-Qntx-Asker-User")
}

#[derive(Clone, Default)]
pub struct SbvhPluginService {
    state: Arc<RwLock<State>>,
    /// Held across reading the calendar and writing a booking, so two people
    /// choosing the same slot in the same second cannot both have it. One
    /// process serves every booking, so one lock is the whole of it.
    booking: Arc<tokio::sync::Mutex<()>>,
}

impl SbvhPluginService {
    pub fn new() -> Self {
        Self::default()
    }

    /// Write one attestation, failure and all. A booking exists only as this
    /// write, so a failure here is the request's failure.
    async fn try_attest(&self, command: crate::attest::Command) -> Result<(), Status> {
        let (endpoint, token) = {
            let s = self.state.read().expect("state lock poisoned");
            (s.ats_endpoint.clone(), s.auth_token.clone())
        };
        if endpoint.is_empty() {
            return Err(Status::failed_precondition("no ats endpoint; not initialized"));
        }

        let mut client = AtsStoreServiceClient::connect(format!("http://{endpoint}"))
            .await
            .map_err(|e| Status::unavailable(format!("ats connect failed: {e}")))?;

        let answered = client
            .generate_and_create_attestation(GenerateAttestationRequest {
                auth_token: token,
                command: Some(crate::attest::to_command(command)),
            })
            .await
            .map_err(|e| Status::unavailable(format!("ats write failed: {e}")))?
            .into_inner();

        if !answered.success {
            return Err(Status::internal(format!("the store refused the write: {}", answered.error)));
        }
        Ok(())
    }

    /// Every attestation carrying one predicate. A failure is an error the
    /// caller sees, never an empty list: no calls and a store that would not
    /// answer are different facts, and the second shown as the first would
    /// offer a slot somebody already has.
    async fn attestations(&self, predicate: &str) -> Result<Vec<Attestation>, Status> {
        let (endpoint, token) = {
            let s = self.state.read().expect("state lock poisoned");
            (s.ats_endpoint.clone(), s.auth_token.clone())
        };
        if endpoint.is_empty() {
            return Err(Status::failed_precondition("no ats endpoint; not initialized"));
        }

        let mut client = AtsStoreServiceClient::connect(format!("http://{endpoint}"))
            .await
            .map_err(|e| Status::unavailable(format!("ats connect failed: {e}")))?;

        let answered = client
            .get_attestations(GetAttestationsRequest {
                auth_token: token,
                filter: Some(AttestationFilter {
                    predicates: vec![predicate.to_string()],
                    contexts: vec![meet::CONTEXT.to_string()],
                    from: Some(attestation_filter::From::UnboundedStart(true)),
                    to: Some(attestation_filter::To::UnboundedEnd(true)),
                    limit: EVERY_ROW,
                    ..Default::default()
                }),
            })
            .await
            .map_err(|e| Status::unavailable(format!("ats read failed: {e}")))?
            .into_inner();

        if !answered.success {
            return Err(Status::internal(format!("the store refused the read: {}", answered.error)));
        }
        // A store may match loosely; the predicate is held to exactly here.
        Ok(answered
            .attestations
            .into_iter()
            .filter(|a| a.predicates.iter().any(|p| p == predicate))
            .collect())
    }

    /// Every call standing now.
    async fn calls(&self, at: i64) -> Result<Vec<Booked>, Status> {
        let booked = self.attestations(meet::BOOKED).await?;
        let cancelled = self.attestations(meet::CANCELLED).await?;
        Ok(standing(&booked, &cancelled, at))
    }

    fn calendar(&self) -> Result<Calendar, String> {
        let s = self.state.read().expect("state lock poisoned");
        match &s.calendar {
            None => Err("no calendar; not initialized".to_string()),
            Some(Err(e)) => Err(format!("the plugin's record names no calendar that can be read: {e}")),
            Some(Ok(c)) => Ok(c.clone()),
        }
    }

    fn place(&self) -> String {
        self.state.read().expect("state lock poisoned").place.clone()
    }

    async fn set_template(&self) -> Result<(), String> {
        let (endpoint, token) = {
            let s = self.state.read().expect("state lock poisoned");
            (reach(s.initialized, &s.mail_endpoint, "mail"), s.auth_token.clone())
        };
        let endpoint = endpoint?;
        let mut client = MailServiceClient::connect(format!("http://{endpoint}"))
            .await
            .map_err(|e| format!("mail connect failed: {e}"))?;
        let answered = client
            .set_template(SetMailTemplateRequest {
                auth_token: token,
                source: PLUGIN.to_string(),
                name: crate::mail::TEMPLATE.to_string(),
                template: Some(crate::mail::template()),
            })
            .await
            .map_err(|e| format!("mail template rpc failed: {e}"))?
            .into_inner();
        if !answered.success {
            return Err(format!("the mail template was not kept: {}", answered.error));
        }
        info!(attestation = %answered.attestation_id, "mail template kept");
        Ok(())
    }

    /// One mail to one User, from the sbvh template.
    async fn mail(&self, user: &str, (subject, body): (String, String)) -> Result<(), String> {
        let (endpoint, token) = {
            let s = self.state.read().expect("state lock poisoned");
            (reach(s.initialized, &s.mail_endpoint, "mail"), s.auth_token.clone())
        };
        let endpoint = endpoint?;
        let mut client = MailServiceClient::connect(format!("http://{endpoint}"))
            .await
            .map_err(|e| format!("mail connect failed: {e}"))?;
        let answered = client
            .send(SendMailRequest {
                auth_token: token,
                source: PLUGIN.to_string(),
                user_id: user.to_string(),
                template: crate::mail::TEMPLATE.to_string(),
                values: HashMap::from([("subject".to_string(), subject.clone()), ("body".to_string(), body)]),
                inline: vec![],
            })
            .await
            .map_err(|e| format!("mail rpc failed: {e}"))?
            .into_inner();
        if !answered.success {
            return Err(answered.error);
        }
        info!(%user, %subject, attestation = %answered.attestation_id, "mailed");
        Ok(())
    }

    /// A mail's outcome as the answer says it: true, or the reason it did not
    /// leave. The booking stands either way, and the browser shows which.
    fn told(sent: Result<(), String>, who: &str) -> serde_json::Value {
        match sent {
            Ok(()) => serde_json::json!(true),
            Err(e) => {
                error!(%who, error = %e, "not mailed");
                serde_json::json!({ "error": e })
            }
        }
    }

    async fn tell_owner(&self, mail: (String, String)) -> serde_json::Value {
        let owner = self.state.read().expect("state lock poisoned").owner.clone();
        match owner {
            None => Self::told(Err(format!("the plugin's record names no {OWNER_KEY} to tell")), "owner"),
            Some(owner) => Self::told(self.mail(&owner, mail).await, "owner"),
        }
    }
}

type Answer = Result<Response<HttpResponse>, Status>;

fn json(code: i32, v: serde_json::Value) -> Answer {
    let (code, body) = match serde_json::to_vec(&v) {
        Ok(b) => (code, b),
        Err(e) => {
            error!(error = %e, "response serialization failed");
            (500, format!(r#"{{"error":"{e}"}}"#).into_bytes())
        }
    };
    Ok(Response::new(HttpResponse {
        status_code: code,
        headers: vec![qntx_grpc::plugin::proto::HttpHeader {
            name: "Content-Type".to_string(),
            values: vec!["application/json".to_string()],
        }],
        body,
    }))
}

fn refused(code: i32, error: impl std::fmt::Display) -> Answer {
    json(code, serde_json::json!({ "error": error.to_string() }))
}

impl SbvhPluginService {
    async fn slots(&self) -> Answer {
        let cal = match self.calendar() {
            Ok(c) => c,
            Err(e) => return refused(503, e),
        };
        let at = now();
        let calls = match self.calls(at).await {
            Ok(c) => c,
            Err(e) => {
                error!(error = %e, "slots could not read the calls");
                return refused(503, e.message());
            }
        };
        let held: Vec<Held> = calls.iter().map(Booked::held).collect();
        json(
            200,
            serde_json::json!({
                "zone":    "Europe/Amsterdam",
                "minutes": cal.minutes,
                "place":   self.place(),
                "slots":   meet::open(at, &cal, &held),
            }),
        )
    }

    async fn book(&self, req: &HttpRequest) -> Answer {
        let Some(user) = asker(req) else {
            return refused(401, "no user");
        };
        let said: serde_json::Value = match serde_json::from_slice(&req.body) {
            Ok(v) => v,
            Err(e) => return refused(400, e),
        };
        let Some(start) = crate::attest::whole(said.get("start")) else {
            return refused(400, "no start: a slot is the unix second /slots named");
        };
        let about = match meet::about(&said["about"]) {
            Ok(a) => a,
            Err(e) => return refused(400, e),
        };
        let replaces = said["replaces"].as_str().filter(|r| !r.is_empty()).map(str::to_string);
        let cal = match self.calendar() {
            Ok(c) => c,
            Err(e) => return refused(503, e),
        };

        // From reading what is held to writing the booking, nobody else books.
        let _held = self.booking.lock().await;
        let at = now();
        let calls = match self.calls(at).await {
            Ok(c) => c,
            Err(e) => {
                error!(error = %e, "book could not read the calls");
                return refused(503, e.message());
            }
        };

        let mine: Vec<&Booked> = calls.iter().filter(|c| c.user == user).collect();
        let moving = match &replaces {
            None => None,
            Some(id) => match mine.iter().find(|c| &c.meet_id == id) {
                Some(c) => Some((*c).clone()),
                None => return refused(404, "you have no call under that meet_id to move"),
            },
        };
        // One call at a time. Somebody who wants another time moves the one
        // they have, which is one mail each way instead of two calls to sort.
        if moving.is_none() {
            if let Some(c) = mine.first() {
                return json(
                    409,
                    serde_json::json!({ "error": "you already have a call booked", "call": c.to_json() }),
                );
            }
        }

        let held: Vec<Held> = calls
            .iter()
            .filter(|c| Some(&c.meet_id) != replaces.as_ref())
            .map(Booked::held)
            .collect();
        if !meet::is_open(start, at, &cal, &held) {
            return refused(409, "that time is no longer open");
        }

        let booked = Booked {
            meet_id: uuid::Uuid::new_v4().to_string(),
            user: user.clone(),
            identity: header(req, "X-Qntx-Asker").unwrap_or_default(),
            start,
            minutes: cal.minutes,
            about,
            name: meet::said(&said["name"]),
            email: meet::said(&said["email"]),
        };
        let mut command = crate::attest::base(&booked.meet_id, meet::CONTEXT, meet::BOOKED);
        command.actors = vec![user.clone()];
        for (key, value) in [
            ("start", serde_json::json!(booked.start)),
            ("minutes", serde_json::json!(booked.minutes)),
            ("about", serde_json::json!(booked.about)),
            ("name", serde_json::json!(booked.name)),
            ("email", serde_json::json!(booked.email)),
            ("identity", serde_json::json!(booked.identity)),
        ] {
            command.attributes.insert(key.to_string(), value);
        }
        if let Some(old) = &moving {
            command.attributes.insert("replaces".to_string(), serde_json::json!(old.meet_id));
        }
        if let Err(e) = self.try_attest(command).await {
            error!(meet_id = %booked.meet_id, error = %e, "book could not be recorded");
            return refused(503, e.message());
        }

        // The new call is written before the old one goes, so a failure in
        // between leaves somebody with two calls rather than none.
        if let Some(old) = &moving {
            let mut command = crate::attest::base(&old.meet_id, meet::CONTEXT, meet::CANCELLED);
            command.actors = vec![user.clone()];
            command.attributes.insert("moved_to".to_string(), serde_json::json!(booked.meet_id));
            if let Err(e) = self.try_attest(command).await {
                error!(meet_id = %old.meet_id, error = %e, "the call moved from could not be cancelled");
                return json(
                    503,
                    serde_json::json!({
                        "error": format!("the new time is booked, and the old one could not be cancelled: {}", e.message()),
                        "call": booked.to_json(),
                    }),
                );
            }
        }
        drop(_held);

        let place = self.place();
        let you = Self::told(self.mail(&user, crate::mail::booked(&booked.call(&place))).await, "prospect");
        let mut notice = crate::mail::arrived(&booked.call(&place), &user, &booked.identity);
        if let Some(old) = &moving {
            notice.0 = format!("Call moved: {}", meet::said_at(booked.start));
            notice.1 = format!("{}\n\nMoved from: {}", notice.1, meet::said_at(old.start));
        }
        let owner = self.tell_owner(notice).await;
        json(200, serde_json::json!({ "call": booked.to_json(), "told": { "you": you, "owner": owner } }))
    }

    async fn mine(&self, req: &HttpRequest) -> Answer {
        let Some(user) = asker(req) else {
            return refused(401, "no user");
        };
        match self.calls(now()).await {
            Ok(calls) => json(
                200,
                serde_json::json!({
                    "calls": calls.iter().filter(|c| c.user == user).map(Booked::to_json).collect::<Vec<_>>(),
                }),
            ),
            Err(e) => {
                error!(error = %e, "mine could not read the calls");
                refused(503, e.message())
            }
        }
    }

    async fn cancel(&self, req: &HttpRequest) -> Answer {
        let Some(user) = asker(req) else {
            return refused(401, "no user");
        };
        let said: serde_json::Value = match serde_json::from_slice(&req.body) {
            Ok(v) => v,
            Err(e) => return refused(400, e),
        };
        let meet_id = match said["meet_id"].as_str().unwrap_or_default() {
            "" => return refused(400, "no meet_id"),
            id => id.to_string(),
        };
        let root = header(req, "X-Qntx-Asker-Level").as_deref() == Some("ROOT");

        let calls = match self.calls(now()).await {
            Ok(c) => c,
            Err(e) => {
                error!(error = %e, "cancel could not read the calls");
                return refused(503, e.message());
            }
        };
        // Somebody else's call answers the same as no call: a meet id is not
        // a way to learn that a call exists.
        let Some(call) = calls.into_iter().find(|c| c.meet_id == meet_id && (root || c.user == user)) else {
            return refused(404, "no call of yours under that meet_id");
        };

        let mut command = crate::attest::base(&call.meet_id, meet::CONTEXT, meet::CANCELLED);
        command.actors = vec![user.clone()];
        if let Err(e) = self.try_attest(command).await {
            error!(%meet_id, error = %e, "cancel could not be recorded");
            return refused(503, e.message());
        }

        let place = self.place();
        let them = Self::told(self.mail(&call.user, crate::mail::cancelled_for_them(&call.call(&place))).await, "prospect");
        let owner = self.tell_owner(crate::mail::cancelled_for_owner(&call.call(&place), &call.user)).await;
        json(200, serde_json::json!({ "meet_id": meet_id, "told": { "you": them, "owner": owner } }))
    }

    async fn all(&self) -> Answer {
        match self.calls(now()).await {
            Ok(calls) => json(200, serde_json::json!({ "calls": calls.iter().map(Booked::to_json_whole).collect::<Vec<_>>() })),
            Err(e) => {
                error!(error = %e, "calls could not be read");
                refused(503, e.message())
            }
        }
    }
}

#[tonic::async_trait]
impl DomainPluginService for SbvhPluginService {
    async fn metadata(&self, _r: Request<Empty>) -> Result<Response<MetadataResponse>, Status> {
        Ok(Response::new(MetadataResponse {
            name: PLUGIN.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            qntx_version: ">=0.1.0".to_string(),
            description: "Booking a call with SBVH".to_string(),
            author: "QNTX Contributors".to_string(),
            license: "AGPL-3.0-or-later".to_string(),
        }))
    }

    async fn initialize(&self, request: Request<InitializeRequest>) -> Result<Response<InitializeResponse>, Status> {
        let req = request.into_inner();
        let calendar = meet::calendar(&req.config);

        info!(
            ats_store = %req.ats_store_endpoint,
            mail = ?req.mail_endpoint,
            namespace = ?req.config.get("namespace"),
            owner = ?req.config.get(OWNER_KEY),
            calendar = ?calendar,
            token_len = req.auth_token.len(),
            "core endpoints"
        );
        if let Err(e) = &calendar {
            error!(error = %e, "the plugin's record names no calendar that can be read");
        }

        {
            let mut state = self.state.write().expect("state lock poisoned");
            state.ats_endpoint = req.ats_store_endpoint;
            state.mail_endpoint = req.mail_endpoint;
            state.auth_token = req.auth_token;
            state.calendar = Some(calendar);
            state.owner = req.config.get(OWNER_KEY).map(|o| o.trim().to_string()).filter(|o| !o.is_empty());
            state.place = req.config.get(PLACE_KEY).map(|p| p.trim().to_string()).unwrap_or_default();
            if state.sentry.is_none() {
                if let Some(dsn) = req.config.get(crate::telemetry::DSN_KEY).filter(|d| !d.is_empty()) {
                    let environment = req.config.get(crate::telemetry::ENVIRONMENT_KEY).map(String::as_str);
                    state.sentry = Some(crate::telemetry::bind(dsn, environment));
                }
            }
            state.initialized = true;
        }

        let kept = self.clone();
        tokio::spawn(async move {
            if let Err(e) = kept.set_template().await {
                error!(error = %e, "the sbvh mail template was not kept");
            }
        });

        // Core exposes what is declared here and nothing else.
        Ok(Response::new(InitializeResponse {
            http_routes: crate::openapi::ROUTES
                .iter()
                .map(|r| RouteInfo {
                    method: r.method.to_string(),
                    path: r.path.to_string(),
                    description: r.summary.to_string(),
                })
                .collect(),
            sigils_from: SigilsFrom::HttpRoutes as i32,
            ..Default::default()
        }))
    }

    async fn health(&self, _r: Request<Empty>) -> Result<Response<HealthResponse>, Status> {
        let state = self.state.read().expect("state lock poisoned");
        let mut details = HashMap::new();
        details.insert("plugin".to_string(), PLUGIN.to_string());
        details.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
        let (healthy, message) = match &state.calendar {
            _ if !state.initialized => (false, "not initialized".to_string()),
            Some(Err(e)) => (false, format!("the calendar cannot be read: {e}")),
            _ => (true, "loaded".to_string()),
        };
        Ok(Response::new(HealthResponse { healthy, message, details }))
    }

    async fn handle_http(&self, request: Request<HttpRequest>) -> Result<Response<HttpResponse>, Status> {
        let req = request.into_inner();
        // A Status returned from here reaches the caller as a bare 502. Without
        // this line nothing says a request even arrived.
        info!(method = %req.method, path = %req.path, body_len = req.body.len(), "http");

        // Core calls this with both the full path and the path stripped of the
        // /api/<plugin> prefix, and either may carry a query.
        let path = req.path.split('?').next().unwrap_or_default().trim_end_matches('/');
        let route = path.strip_prefix("/api/sbvhAPI").unwrap_or(path);

        match (req.method.as_str(), route) {
            ("GET", "/openapi.json") => json(200, crate::openapi::document()),
            ("GET", "/health" | "") => {
                let (initialized, calendar) = {
                    let s = self.state.read().expect("state lock poisoned");
                    (s.initialized, s.calendar.clone())
                };
                json(
                    200,
                    serde_json::json!({
                        "plugin":      PLUGIN,
                        "version":     env!("CARGO_PKG_VERSION"),
                        "initialized": initialized,
                        "calendar":    match calendar {
                            None => serde_json::Value::Null,
                            Some(Ok(_)) => serde_json::json!("ok"),
                            Some(Err(e)) => serde_json::json!({ "error": e }),
                        },
                    }),
                )
            }
            ("GET", "/slots") => self.slots().await,
            ("POST", "/book") => self.book(&req).await,
            ("GET", "/mine") => self.mine(&req).await,
            ("POST", "/cancel") => self.cancel(&req).await,
            ("GET", "/calls") => self.all().await,
            (method, _) => refused(404, format!("{PLUGIN} serves no {method} {path}")),
        }
    }

    async fn execute_job(&self, request: Request<ExecuteJobRequest>) -> Result<Response<ExecuteJobResponse>, Status> {
        let req = request.into_inner();
        error!(job = %req.job_id, handler = %req.handler_name, "job for a handler this plugin does not declare");
        Ok(Response::new(ExecuteJobResponse {
            success: false,
            error: format!("{PLUGIN} declares no handler '{}'", req.handler_name),
            plugin_version: env!("CARGO_PKG_VERSION").to_string(),
            ..Default::default()
        }))
    }

    type HandleWebSocketStream = ReceiverStream<Result<WebSocketMessage, Status>>;

    async fn handle_web_socket(
        &self,
        _r: Request<tonic::Streaming<WebSocketMessage>>,
    ) -> Result<Response<Self::HandleWebSocketStream>, Status> {
        Err(Status::unimplemented("no websocket surface"))
    }

    async fn shutdown(&self, _r: Request<Empty>) -> Result<Response<ShutdownResponse>, Status> {
        info!("{PLUGIN} shutting down");
        Ok(Response::new(ShutdownResponse { stopped: true }))
    }

    // The keys Initialize reads off the plugin's record (ADR-043). While the
    // plugin runs, the node refuses a key the schema does not name.
    async fn config_schema(&self, _r: Request<Empty>) -> Result<Response<ConfigSchemaResponse>, Status> {
        let field = |kind: &str, description: &str, default: &str| ConfigFieldSchema {
            r#type: kind.to_string(),
            description: description.to_string(),
            default_value: default.to_string(),
            ..Default::default()
        };
        Ok(Response::new(ConfigSchemaResponse {
            fields: HashMap::from([
                (OWNER_KEY.to_string(), field("string", "The User told of every call booked and cancelled", "")),
                (PLACE_KEY.to_string(), field("string", "Where the call happens, as the mails say it", "")),
                (
                    "hours".to_string(),
                    field("string", "Open hours in Amsterdam time: mon-fri 10:00-12:00,14:00-17:00; sat 10:00-12:00", meet::DEFAULT_HOURS),
                ),
                ("slot_minutes".to_string(), field("integer", "How long one call is", &meet::DEFAULT_MINUTES.to_string())),
                ("notice_hours".to_string(), field("integer", "How far ahead the first bookable slot is", &meet::DEFAULT_NOTICE_HOURS.to_string())),
                ("horizon_days".to_string(), field("integer", "How many days ahead slots are offered", &meet::DEFAULT_HORIZON_DAYS.to_string())),
                (crate::telemetry::DSN_KEY.to_string(), field("string", "Where this plugin ships its log and its errors", "")),
                (crate::telemetry::ENVIRONMENT_KEY.to_string(), field("string", "The Sentry environment those go under", "")),
            ]),
        }))
    }

    async fn register_elements(&self, _r: Request<Empty>) -> Result<Response<ElementDefResponse>, Status> {
        Ok(Response::new(ElementDefResponse { elements: vec![] }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qntx_grpc::plugin::proto::HttpHeader;

    async fn ask(svc: &SbvhPluginService, method: &str, path: &str, user: Option<&str>, body: serde_json::Value) -> (i32, serde_json::Value) {
        let headers = user
            .map(|u| vec![HttpHeader { name: "X-Qntx-Asker-User".to_string(), values: vec![u.to_string()] }])
            .unwrap_or_default();
        let answered = svc
            .handle_http(Request::new(HttpRequest {
                method: method.to_string(),
                path: path.to_string(),
                headers,
                body: serde_json::to_vec(&body).unwrap(),
            }))
            .await
            .expect("handle_http answers rather than erroring")
            .into_inner();
        (answered.status_code, serde_json::from_slice(&answered.body).expect("every answer is JSON"))
    }

    async fn initialized(config: &[(&str, &str)]) -> SbvhPluginService {
        let svc = SbvhPluginService::new();
        svc.initialize(Request::new(InitializeRequest {
            config: config.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            ..Default::default()
        }))
        .await
        .expect("initializes");
        svc
    }

    fn attestation(subject: &str, predicate: &str, actor: &str, attributes: serde_json::Value) -> Attestation {
        let mut c = crate::attest::base(subject, meet::CONTEXT, predicate);
        if let serde_json::Value::Object(o) = attributes {
            c.attributes = o;
        }
        let command = crate::attest::to_command(c);
        Attestation {
            subjects: vec![subject.to_string()],
            predicates: vec![predicate.to_string()],
            actors: vec![actor.to_string()],
            attributes: command.attributes,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn initialize_declares_every_route_the_document_names() {
        let svc = SbvhPluginService::new();
        let answered = svc.initialize(Request::new(InitializeRequest::default())).await.unwrap().into_inner();
        let declared: Vec<(String, String)> = answered.http_routes.iter().map(|r| (r.method.clone(), r.path.clone())).collect();
        for r in crate::openapi::ROUTES {
            assert!(declared.contains(&(r.method.to_string(), r.path.to_string())), "{} {} is not declared", r.method, r.path);
        }
        assert_eq!(answered.sigils_from, SigilsFrom::HttpRoutes as i32);
    }

    #[tokio::test]
    async fn every_route_the_document_names_is_answered() {
        let svc = initialized(&[]).await;
        for r in crate::openapi::ROUTES {
            let (code, said) = ask(&svc, r.method, &format!("/api/sbvhAPI{}", r.path), Some("U-1"), serde_json::json!({})).await;
            assert_ne!(code, 404, "{} {} fell through to the 404 arm: {said}", r.method, r.path);
        }
    }

    #[tokio::test]
    async fn a_route_is_answered_with_or_without_the_prefix() {
        let svc = initialized(&[]).await;
        let (full, _) = ask(&svc, "GET", "/api/sbvhAPI/openapi.json", None, serde_json::json!({})).await;
        let (bare, _) = ask(&svc, "GET", "/openapi.json", None, serde_json::json!({})).await;
        assert_eq!((full, bare), (200, 200));
    }

    #[tokio::test]
    async fn an_unknown_route_is_a_404_and_says_so() {
        let svc = initialized(&[]).await;
        let (code, said) = ask(&svc, "GET", "/api/sbvhAPI/nothing", None, serde_json::json!({})).await;
        assert_eq!(code, 404);
        assert!(said["error"].as_str().unwrap().contains("/nothing"));
    }

    #[tokio::test]
    async fn booking_without_a_user_is_a_401() {
        let svc = initialized(&[]).await;
        for (method, path) in [("POST", "/book"), ("GET", "/mine"), ("POST", "/cancel")] {
            let (code, _) = ask(&svc, method, path, None, serde_json::json!({})).await;
            assert_eq!(code, 401, "{method} {path}");
        }
    }

    #[tokio::test]
    async fn booking_without_a_start_says_which_start_it_wants() {
        let svc = initialized(&[]).await;
        let (code, said) = ask(&svc, "POST", "/book", Some("U-1"), serde_json::json!({ "about": "hi" })).await;
        assert_eq!(code, 400);
        assert!(said["error"].as_str().unwrap().contains("/slots"), "{said}");
    }

    #[tokio::test]
    async fn too_much_written_is_refused_before_anything_is_read() {
        let svc = initialized(&[]).await;
        let long = "x".repeat(meet::ABOUT_MOST + 1);
        let (code, _) = ask(&svc, "POST", "/book", Some("U-1"), serde_json::json!({ "start": 1, "about": long })).await;
        assert_eq!(code, 400);
    }

    #[tokio::test]
    async fn cancel_without_a_meet_id_says_so() {
        let svc = initialized(&[]).await;
        let (code, said) = ask(&svc, "POST", "/cancel", Some("U-1"), serde_json::json!({})).await;
        assert_eq!((code, said["error"].as_str()), (400, Some("no meet_id")));
    }

    #[tokio::test]
    async fn without_a_store_the_slots_are_refused_rather_than_all_offered() {
        // No calls and a store that never answered are different facts: the
        // second shown as the first would offer slots somebody already has.
        let svc = initialized(&[]).await;
        let (code, said) = ask(&svc, "GET", "/slots", Some("U-1"), serde_json::json!({})).await;
        assert_eq!(code, 503);
        assert!(said["error"].as_str().unwrap().contains("ats"), "{said}");
    }

    #[tokio::test]
    async fn a_record_with_unreadable_hours_says_so_at_slots_and_in_health() {
        let svc = initialized(&[("hours", "whenever")]).await;
        let (code, said) = ask(&svc, "GET", "/slots", Some("U-1"), serde_json::json!({})).await;
        assert_eq!(code, 503);
        assert!(said["error"].as_str().unwrap().contains("hours"), "{said}");
        let health = svc.health(Request::new(Empty {})).await.unwrap().into_inner();
        assert!(!health.healthy);
        assert!(health.message.contains("calendar"), "{}", health.message);
    }

    #[tokio::test]
    async fn health_before_initialize_says_it_is_not_initialized() {
        let health = SbvhPluginService::new().health(Request::new(Empty {})).await.unwrap().into_inner();
        assert_eq!((health.healthy, health.message.as_str()), (false, "not initialized"));
    }

    #[test]
    fn a_cancelled_or_ended_call_no_longer_stands() {
        let at = 1_000_000;
        let booked = vec![
            attestation("M-past", meet::BOOKED, "U-1", serde_json::json!({ "start": at - 3_600, "minutes": 30 })),
            attestation("M-gone", meet::BOOKED, "U-1", serde_json::json!({ "start": at + 3_600, "minutes": 30 })),
            attestation("M-now", meet::BOOKED, "U-2", serde_json::json!({ "start": at - 600, "minutes": 30 })),
            attestation("M-later", meet::BOOKED, "U-3", serde_json::json!({ "start": at + 7_200, "minutes": 30, "about": "hi" })),
        ];
        let cancelled = vec![attestation("M-gone", meet::CANCELLED, "U-1", serde_json::json!({}))];
        let ids: Vec<String> = standing(&booked, &cancelled, at).into_iter().map(|b| b.meet_id).collect();
        assert_eq!(ids, vec!["M-now", "M-later"]);
    }

    #[test]
    fn a_booking_the_store_cannot_place_in_time_is_left_out_rather_than_guessed() {
        let booked = vec![attestation("M-1", meet::BOOKED, "U-1", serde_json::json!({ "minutes": 30 }))];
        assert!(standing(&booked, &[], 0).is_empty());
    }

    #[test]
    fn a_service_the_node_runs_without_is_refused_with_the_node_s_reason() {
        let not_running = Some(ServiceEndpoint { state: Some(service_endpoint::State::NotRunning("no SES".into())) });
        assert!(reach(true, &not_running, "mail").unwrap_err().contains("no SES"));
        assert!(reach(false, &not_running, "mail").unwrap_err().contains("not initialized"));
    }

    #[tokio::test]
    async fn the_schema_names_every_key_initialize_reads() {
        let schema = SbvhPluginService::new().config_schema(Request::new(Empty {})).await.unwrap().into_inner();
        for key in [OWNER_KEY, PLACE_KEY, "hours", "slot_minutes", "notice_hours", "horizon_days", "sentry_dsn", "sentry_environment"] {
            assert!(schema.fields.contains_key(key), "{key} is read and not in the schema");
        }
    }
}
