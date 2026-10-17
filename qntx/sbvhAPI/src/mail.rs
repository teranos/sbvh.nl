//! The one template every SBVH mail is filled from, and what each mail says.
//! QNTX fills the template with Go templates: subject and text with
//! text/template, html with html/template, which escapes what it is given.

use qntx_grpc::plugin::proto::{mail_template_body, MailTemplate, MailTemplateBody};

pub const TEMPLATE: &str = "sbvh";

pub fn template() -> MailTemplate {
    MailTemplate {
        subject: "{{.subject}}".to_string(),
        bodies: vec![
            MailTemplateBody {
                is: Some(mail_template_body::Is::Html(
                    concat!(
                        "<!doctype html><html><body style=\"margin:0;padding:24px;background:#fcffff;color:#333433;",
                        "font-family:'JetBrains Mono','SF Mono',Menlo,Consolas,monospace;font-size:14px;line-height:1.6\">",
                        "<div style=\"max-width:560px\">",
                        "<p style=\"font-family:Georgia,'Times New Roman',serif;font-size:15px;letter-spacing:0.05em;color:#656766\">SBvH</p>",
                        "<h1 style=\"font-family:Georgia,'Times New Roman',serif;font-weight:400;font-size:20px;margin:16px 0\">{{.subject}}</h1>",
                        "<div style=\"white-space:pre-line\">{{.body}}</div>",
                        "<p style=\"margin-top:32px;color:#989a99;font-size:12px\">sbvh.nl</p>",
                        "</div></body></html>"
                    )
                    .to_string(),
                )),
            },
            MailTemplateBody {
                is: Some(mail_template_body::Is::Text("{{.body}}\n\n— Sebastiaan van Houten · sbvh.nl\n".to_string())),
            },
        ],
    }
}

/// What one booking is, as both mails say it.
pub struct Call<'a> {
    pub start: i64,
    pub minutes: i64,
    pub about: &'a str,
    pub name: &'a str,
    pub email: &'a str,
    pub place: &'a str,
}

fn when(c: &Call) -> String {
    format!(
        "{}–{} (Amsterdam time)",
        crate::meet::said_at(c.start),
        crate::meet::said_until(c.start + c.minutes * 60)
    )
}

fn who(c: &Call, user: &str) -> String {
    match (c.name.is_empty(), c.email.is_empty()) {
        (false, false) => format!("{} <{}>", c.name, c.email),
        (false, true) => c.name.to_string(),
        (true, false) => c.email.to_string(),
        (true, true) => format!("QNTX User {user}"),
    }
}

/// What the prospect is told once the call is theirs.
pub fn booked(c: &Call) -> (String, String) {
    let mut lines = vec![
        format!("Thanks{}. Your call with Sebastiaan is booked.", if c.name.is_empty() { String::new() } else { format!(", {}", c.name) }),
        String::new(),
        format!("When: {}", when(c)),
    ];
    if !c.place.is_empty() {
        lines.push(format!("Where: {}", c.place));
    }
    if !c.about.is_empty() {
        lines.extend([String::new(), "You wrote:".to_string(), c.about.to_string()]);
    }
    lines.extend([
        String::new(),
        "To move or cancel it, sign in at sbvh.nl and open Schedule a time.".to_string(),
    ]);
    (format!("Your call on {}", crate::meet::said_at(c.start)), lines.join("\n"))
}

/// What the owner is told when a call is booked.
pub fn arrived(c: &Call, user: &str, identity: &str) -> (String, String) {
    let mut lines = vec![
        format!("{} booked a call.", who(c, user)),
        String::new(),
        format!("When: {}", when(c)),
    ];
    if !c.about.is_empty() {
        lines.extend([String::new(), "They wrote:".to_string(), c.about.to_string()]);
    }
    lines.extend([String::new(), format!("QNTX User: {user}")]);
    if !identity.is_empty() {
        lines.push(format!("Signed in as: {identity}"));
    }
    (format!("Call booked: {}", crate::meet::said_at(c.start)), lines.join("\n"))
}

/// What the prospect is told when their call is cancelled, by them or not.
pub fn cancelled_for_them(c: &Call) -> (String, String) {
    let body = format!(
        "Your call on {} is cancelled.\n\nTo book another time, sign in at sbvh.nl and open Schedule a time.",
        when(c)
    );
    (format!("Cancelled: your call on {}", crate::meet::said_at(c.start)), body)
}

/// What the owner is told when a call is cancelled.
pub fn cancelled_for_owner(c: &Call, user: &str) -> (String, String) {
    let body = format!("The call with {} on {} is cancelled.", who(c, user), when(c));
    (format!("Call cancelled: {}", crate::meet::said_at(c.start)), body)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Monday 19 October 2026, 10:00 Amsterdam.
    const START: i64 = 1_792_396_800;

    fn call<'a>(name: &'a str, email: &'a str, about: &'a str) -> Call<'a> {
        Call { start: START, minutes: 30, about, name, email, place: "Video call, link to follow" }
    }

    #[test]
    fn the_prospect_is_told_when_and_where_and_what_they_wrote() {
        let (subject, body) = booked(&call("Ada", "ada@example.com", "DORA audit in Q1"));
        assert_eq!(subject, "Your call on Monday 19 October 2026, 10:00");
        assert!(body.starts_with("Thanks, Ada."), "{body}");
        assert!(body.contains("When: Monday 19 October 2026, 10:00–10:30 (Amsterdam time)"), "{body}");
        assert!(body.contains("Where: Video call, link to follow"), "{body}");
        assert!(body.contains("DORA audit in Q1"), "{body}");
    }

    #[test]
    fn the_owner_is_told_who_and_by_which_account() {
        let (_, body) = arrived(&call("Ada", "ada@example.com", ""), "U-1", "google:1");
        assert!(body.starts_with("Ada <ada@example.com> booked a call."), "{body}");
        assert!(body.contains("QNTX User: U-1"), "{body}");
        assert!(body.contains("Signed in as: google:1"), "{body}");
        assert!(!body.contains("They wrote"), "nothing written is not a heading over nothing: {body}");
    }

    #[test]
    fn a_prospect_who_handed_over_nothing_is_named_by_their_user() {
        let (_, body) = arrived(&call("", "", ""), "U-1", "");
        assert!(body.starts_with("QNTX User U-1 booked a call."), "{body}");
    }
}
