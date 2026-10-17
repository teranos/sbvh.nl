//! The attestation model.
//!
//! Nothing here talks to the network — these are the pure translations between
//! an attestation's attributes and what this plugin does, so they can be
//! pinned by tests rather than by watching a box.

use qntx_grpc::plugin::proto::{attestation_command, ActorNames, AttestationCommand};

/// A command to write, in the shape ATSStoreService takes.
#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    pub subjects: Vec<String>,
    pub predicates: Vec<String>,
    pub contexts: Vec<String>,
    /// Empty is the node authoring it.
    pub actors: Vec<String>,
    pub attributes: serde_json::Map<String, serde_json::Value>,
}

pub fn base(subject: &str, context: &str, predicate: &str) -> Command {
    Command {
        subjects: vec![subject.to_string()],
        predicates: vec![predicate.to_string()],
        contexts: if context.is_empty() {
            vec![]
        } else {
            vec![context.to_string()]
        },
        actors: vec![],
        attributes: serde_json::Map::new(),
    }
}

pub fn to_command(c: Command) -> AttestationCommand {
    let fields = c
        .attributes
        .into_iter()
        .map(|(k, v)| (k, json_to_value(v)))
        .collect();

    let by = if c.actors.is_empty() {
        attestation_command::By::ByNode(true)
    } else {
        attestation_command::By::ActorsNamed(ActorNames { names: c.actors })
    };

    AttestationCommand {
        subjects: c.subjects,
        predicates: c.predicates,
        contexts: c.contexts,
        by: Some(by),
        when: Some(attestation_command::When::Now(true)),
        attributes: Some(prost_types::Struct { fields }),
        source: crate::service::PLUGIN.to_string(),
        version: Some(attestation_command::Version::SourceVersion(
            env!("CARGO_PKG_VERSION").to_string(),
        )),
    }
}

fn json_to_value(v: serde_json::Value) -> prost_types::Value {
    use prost_types::value::Kind;
    let kind = match v {
        serde_json::Value::Null => Kind::NullValue(0),
        serde_json::Value::Bool(b) => Kind::BoolValue(b),
        serde_json::Value::Number(n) => Kind::NumberValue(n.as_f64().unwrap_or(f64::NAN)),
        serde_json::Value::String(s) => Kind::StringValue(s),
        serde_json::Value::Array(a) => Kind::ListValue(prost_types::ListValue {
            values: a.into_iter().map(json_to_value).collect(),
        }),
        serde_json::Value::Object(o) => Kind::StructValue(prost_types::Struct {
            fields: o.into_iter().map(|(k, v)| (k, json_to_value(v))).collect(),
        }),
    };
    prost_types::Value { kind: Some(kind) }
}

pub fn attributes_of(
    a: &qntx_grpc::plugin::proto::Attestation,
) -> serde_json::Map<String, serde_json::Value> {
    a.attributes
        .as_ref()
        .map(|s| {
            s.fields
                .iter()
                .map(|(k, v)| (k.clone(), value_to_json(v)))
                .collect()
        })
        .unwrap_or_default()
}

fn value_to_json(v: &prost_types::Value) -> serde_json::Value {
    use prost_types::value::Kind;
    match &v.kind {
        None | Some(Kind::NullValue(_)) => serde_json::Value::Null,
        Some(Kind::BoolValue(b)) => (*b).into(),
        Some(Kind::NumberValue(n)) => serde_json::Number::from_f64(*n)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Some(Kind::StringValue(s)) => s.clone().into(),
        Some(Kind::ListValue(l)) => l.values.iter().map(value_to_json).collect(),
        Some(Kind::StructValue(s)) => s
            .fields
            .iter()
            .map(|(k, v)| (k.clone(), value_to_json(v)))
            .collect::<serde_json::Map<_, _>>()
            .into(),
    }
}

/// A whole number an attribute carries. The store keeps every number as a
/// double, so 1792000000 comes back as 1792000000.0 and `as_i64` says None.
pub fn whole(v: Option<&serde_json::Value>) -> Option<i64> {
    let n = v?.as_f64()?;
    if n.fract() != 0.0 || !n.is_finite() {
        return None;
    }
    Some(n as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_number_round_trips_through_the_store_as_a_whole_number() {
        let mut c = base("M-1", "meet", "meet-book");
        c.attributes.insert("start".into(), 1_792_000_000_i64.into());
        let command = to_command(c);
        let stored = qntx_grpc::plugin::proto::Attestation {
            attributes: command.attributes,
            ..Default::default()
        };
        let read = attributes_of(&stored);
        assert_eq!(read["start"].as_i64(), None, "the store hands back a double");
        assert_eq!(whole(read.get("start")), Some(1_792_000_000));
    }

    #[test]
    fn a_fraction_is_not_a_whole_number() {
        assert_eq!(whole(Some(&serde_json::json!(1.5))), None);
        assert_eq!(whole(Some(&serde_json::json!("1"))), None);
        assert_eq!(whole(None), None);
    }

    #[test]
    fn a_command_by_a_user_names_them_and_one_without_is_the_node() {
        let mut c = base("M-1", "meet", "meet-book");
        assert!(matches!(to_command(c.clone()).by, Some(attestation_command::By::ByNode(true))));
        c.actors = vec!["user-1".into()];
        match to_command(c).by {
            Some(attestation_command::By::ActorsNamed(n)) => assert_eq!(n.names, vec!["user-1"]),
            other => panic!("expected the user named, got {other:?}"),
        }
    }
}
