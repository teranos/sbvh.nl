//! What this build serves, in the form a machine reads.
//!
//! The document is rendered from `ROUTES`, the same table `handle_http`
//! dispatches from and Initialize declares, so a route and its description
//! cannot disagree.

use serde_json::json;

/// One route, as the dispatcher, Initialize and the document all see it.
pub struct Route {
    pub method: &'static str,
    /// Relative to `/api/sbvhAPI`, the prefix core mounts this plugin at.
    pub path: &'static str,
    pub summary: &'static str,
    /// Who the node has to be told reaches it, as a reach line says it. ROOT
    /// is the node's default for a route no line opens, so it needs none.
    pub reach: &'static str,
}

pub const ROUTES: &[Route] = &[
    Route { method: "GET", path: "/openapi.json", summary: "What this build serves.", reach: "ROOT" },
    Route { method: "GET", path: "/health", summary: "Whether the plugin loaded and reads its calendar.", reach: "ROOT" },
    Route {
        method: "GET",
        path: "/slots",
        summary: "The times a call can be booked at, as unix seconds, earliest first.",
        reach: "PUBLIC_REGISTRATION",
    },
    Route {
        method: "POST",
        path: "/book",
        summary: "Book one open slot as the asking User. Takes start, and optionally about, name and email.",
        reach: "PUBLIC_REGISTRATION",
    },
    Route {
        method: "GET",
        path: "/mine",
        summary: "The asking User's calls that have not ended, cancelled ones left out.",
        reach: "PUBLIC_REGISTRATION",
    },
    Route {
        method: "POST",
        path: "/cancel",
        summary: "Cancel a call by its meet_id: the asking User's own, or any for ROOT.",
        reach: "PUBLIC_REGISTRATION",
    },
    Route {
        method: "GET",
        path: "/calls",
        summary: "Every call that has not ended, with who booked it.",
        reach: "ROOT",
    },
];

pub fn document() -> serde_json::Value {
    let mut paths = serde_json::Map::new();
    for r in ROUTES {
        let entry = paths
            .entry(format!("/api/{}{}", crate::service::PLUGIN, r.path))
            .or_insert_with(|| json!({}));
        entry[r.method.to_lowercase()] = json!({ "summary": r.summary, "x-qntx-reach": r.reach });
    }
    json!({
        "openapi": "3.1.0",
        "info": { "title": crate::service::PLUGIN, "version": env!("CARGO_PKG_VERSION") },
        "paths": paths,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_route_is_in_the_document_once() {
        let doc = document();
        let paths = doc["paths"].as_object().expect("paths");
        let mut counted = 0;
        for (_, methods) in paths {
            counted += methods.as_object().expect("methods").len();
        }
        assert_eq!(counted, ROUTES.len());
        assert_eq!(doc["paths"]["/api/sbvhAPI/book"]["post"]["x-qntx-reach"], "PUBLIC_REGISTRATION");
    }
}
