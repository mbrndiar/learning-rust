use std::collections::BTreeSet;
use std::sync::Arc;

use serde_json::Value;
use tasks_solution::api::boundary::{HttpBoundary, MAX_BODY_BYTES, StderrReporter};
use tasks_solution::{
    Task, TaskApplication, TaskFilter, TaskPatch, TaskRepository, TaskResult, TaskService,
};

const OPENAPI: &str = include_str!("../../docs/openapi.yaml");

#[test]
fn checked_in_openapi_is_typed_local_and_complete() {
    let _: openapiv3::OpenAPI = serde_yaml_ng::from_str(OPENAPI).expect("parse OpenAPI 3 document");
    let document: Value = serde_yaml_ng::from_str(OPENAPI).expect("inspect OpenAPI document");

    assert_eq!(document["openapi"], "3.1.0");
    let paths = document["paths"].as_object().expect("paths object");
    assert_eq!(
        paths.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        BTreeSet::from(["/health", "/tasks", "/tasks/{taskId}"])
    );
    let http_methods = BTreeSet::from([
        "get", "put", "post", "delete", "options", "head", "patch", "trace",
    ]);
    assert_eq!(
        paths
            .values()
            .filter_map(Value::as_object)
            .flat_map(|path| path.keys())
            .filter(|method| http_methods.contains(method.as_str()))
            .count(),
        6
    );

    let operations = [
        (
            "/health",
            "get",
            "getHealth",
            ["200", "405", "500"].as_slice(),
        ),
        (
            "/tasks",
            "get",
            "listTasks",
            ["200", "405", "422", "500"].as_slice(),
        ),
        (
            "/tasks",
            "post",
            "createTask",
            ["201", "400", "405", "413", "415", "422", "500"].as_slice(),
        ),
        (
            "/tasks/{taskId}",
            "get",
            "getTask",
            ["200", "404", "405", "422", "500"].as_slice(),
        ),
        (
            "/tasks/{taskId}",
            "patch",
            "updateTask",
            ["200", "400", "404", "405", "413", "415", "422", "500"].as_slice(),
        ),
        (
            "/tasks/{taskId}",
            "delete",
            "deleteTask",
            ["204", "404", "405", "422", "500"].as_slice(),
        ),
    ];
    for (path, method, operation_id, statuses) in operations {
        let operation = &document["paths"][path][method];
        assert!(operation.is_object(), "{method} {path}");
        assert_eq!(
            operation["operationId"], operation_id,
            "{method} {path} operationId"
        );
        assert_eq!(
            operation["responses"]
                .as_object()
                .expect("responses object")
                .keys()
                .map(String::as_str)
                .collect::<BTreeSet<_>>(),
            statuses.iter().copied().collect(),
            "{method} {path} statuses"
        );
    }

    assert_eq!(
        document["components"]["schemas"]["Task"]["required"],
        serde_json::json!(["id", "title", "completed"])
    );
    assert_eq!(
        document["components"]["schemas"]["Health"]["required"],
        serde_json::json!(["status"])
    );
    assert_eq!(
        document["components"]["schemas"]["CreateTask"]["required"],
        serde_json::json!(["title"])
    );
    assert_eq!(
        document["components"]["schemas"]["UpdateTask"]["minProperties"],
        1
    );
    assert_eq!(
        document["components"]["schemas"]["Error"]["required"],
        serde_json::json!(["error"])
    );
    assert_eq!(
        document["components"]["schemas"]["Error"]["properties"]["error"]["properties"]["code"]["enum"],
        serde_json::json!([
            "invalid_json",
            "payload_too_large",
            "unsupported_media_type",
            "not_found",
            "method_not_allowed",
            "validation_error",
            "internal_error"
        ])
    );
    assert_eq!(
        document["components"]["responses"]["PayloadTooLarge"]["content"]["application/json"]["examples"]
            ["payloadTooLarge"]["value"]["error"]["code"],
        "payload_too_large"
    );
    assert_eq!(
        document["components"]["responses"]["UnsupportedMediaType"]["content"]["application/json"]
            ["examples"]["unsupportedMediaType"]["value"]["error"]["code"],
        "unsupported_media_type"
    );
    assert_eq!(
        document["components"]["schemas"]["Task"]["properties"]["title"]["maxLength"],
        120
    );
    assert_eq!(
        document["components"]["parameters"]["TaskId"]["schema"]["minimum"],
        1
    );
    assert_eq!(
        document["paths"]["/tasks"]["get"]["parameters"][0]["schema"]["type"],
        "boolean"
    );
    assert_eq!(
        document["paths"]["/tasks/{taskId}"]["delete"]["responses"]["204"]["description"],
        "The task was deleted. The response has no body."
    );

    assert_local_references(&document, &document);
}

fn assert_local_references(root: &Value, value: &Value) {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                if key == "$ref" {
                    let reference = value.as_str().expect("$ref string");
                    assert!(
                        reference.starts_with("#/"),
                        "external reference is not allowed: {reference}"
                    );
                    assert!(
                        root.pointer(&reference[1..]).is_some(),
                        "unresolved local reference: {reference}"
                    );
                }
                assert_local_references(root, value);
            }
        }
        Value::Array(values) => {
            for value in values {
                assert_local_references(root, value);
            }
        }
        _ => {}
    }
}

struct ContractRepository;

impl TaskRepository for ContractRepository {
    fn create(&self, title: &str) -> TaskResult<Task> {
        Task::from_parts(1, title, false)
    }

    fn list(&self, _filter: TaskFilter) -> TaskResult<Vec<Task>> {
        Ok(Vec::new())
    }

    fn get(&self, id: i64) -> TaskResult<Task> {
        Task::from_parts(id, "contract task", false)
    }

    fn update(&self, id: i64, patch: TaskPatch) -> TaskResult<Task> {
        Task::from_parts(
            id,
            patch.title.as_deref().unwrap_or("contract task"),
            patch.completed.unwrap_or(false),
        )
    }

    fn delete(&self, _id: i64) -> TaskResult<()> {
        Ok(())
    }
}

#[tokio::test]
async fn representative_boundary_responses_match_openapi_successes() {
    let document: Value = serde_yaml_ng::from_str(OPENAPI).expect("inspect OpenAPI document");
    let boundary = HttpBoundary::new(
        TaskApplication::new(TaskService::new(Arc::new(ContractRepository))),
        Arc::new(StderrReporter),
    );

    let health = boundary.health(None).await;
    assert_eq!(health.status, 200);
    assert!(
        document["paths"]["/health"]["get"]["responses"][health.status.to_string()].is_object()
    );

    let created = boundary
        .create(
            None,
            Some("application/json"),
            br#"{"title":"contract task"}"#,
        )
        .await;
    assert_eq!(created.status, 201);
    assert!(
        document["paths"]["/tasks"]["post"]["responses"][created.status.to_string()].is_object()
    );

    let oversized_body = vec![b' '; MAX_BODY_BYTES + 1];
    let oversized = boundary
        .create(None, Some("application/json"), &oversized_body)
        .await;
    assert_eq!(oversized.status, 413);
    assert!(
        document["paths"]["/tasks"]["post"]["responses"][oversized.status.to_string()].is_object()
    );

    let unsupported = boundary.create(None, Some("text/plain"), b"{}").await;
    assert_eq!(unsupported.status, 415);
    assert!(
        document["paths"]["/tasks"]["post"]["responses"][unsupported.status.to_string()]
            .is_object()
    );

    let deleted = boundary.delete("1", None).await;
    assert_eq!(deleted.status, 204);
    assert!(deleted.body.is_empty());
    assert!(
        document["paths"]["/tasks/{taskId}"]["delete"]["responses"][deleted.status.to_string()]
            .is_object()
    );
}
