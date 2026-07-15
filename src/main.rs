use axum::{Router, extract::Query, routing::get};
use serde::{Deserialize, Serialize};
use serial_test::serial;
use std::process::Output;
use std::{collections::HashMap, time::Duration};
use thiserror::Error;
use tokio::process::Command;

#[derive(Serialize, Deserialize, Debug)]
struct ScaleResponse {
    success: bool,
    failure_msg: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
struct ScaleRequest {
    replica_count: u32,
}
#[derive(Serialize, Deserialize, Debug)]
struct RestartResponse {
    success: bool,
    failure_msg: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct GetPodsResponse {
    pods: Vec<PodInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PodInfo {
    name: String,
    ip_address: String,
    restart_count: u32,
    is_canary: bool,
    image: String,
    created_at: String,
    status: PodStatus,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum PodStatus {
    Running,
    Starting,
    Terminating,
}

#[derive(Serialize, Deserialize, Debug)]
struct DeployCanaryRequest {
    sha: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DeployCanaryResponse {
    success: bool,
    failure_msg: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PromoteCanaryRequest {
    sha: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct PromoteCanaryResponse {
    success: bool,
    failure_msg: Option<String>,
}

async fn root() -> &'static str {
    return "Hello World";
}

async fn test(Query(params): Query<HashMap<String, String>>) -> String {
    let param = params.get("param").map_or("unknown", String::as_str);
    println!("Received request, {}!", param);
    tokio::time::sleep(Duration::from_secs(1)).await;
    return format!("Processed request, {}!", param);
}

async fn scale(request: ScaleRequest) -> ScaleResponse {
    // kubectl scale deployment/captain --replicas=5
    let replicas = format!("--replicas={}", request.replica_count);
    let output = run_k8_command("kubectl", &["scale", "deployment/captain", &replicas]).await;

    let success = output.status.success();
    let failure_msg = if success {
        None
    } else {
        Some(String::from_utf8_lossy(&output.stderr).into_owned())
    };

    ScaleResponse {
        success,
        failure_msg,
    }
}

async fn restart() -> RestartResponse {
    // kubectl rollout restart deployment/captain
    let output = run_k8_command("kubectl", &["rollout", "restart", "deployment/captain"]).await;

    let success = output.status.success();
    let failure_msg = if success {
        None
    } else {
        Some(String::from_utf8_lossy(&output.stderr).into_owned())
    };

    RestartResponse {
        success,
        failure_msg,
    }
}

#[derive(Error, Debug)]
enum ManifestError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

async fn deploy_canary(request: DeployCanaryRequest) -> DeployCanaryResponse {
    let manifest = "k8/deployment-canary.yaml";

    if let Err(e) = update_manifest(&request.sha, manifest).await {
        return DeployCanaryResponse {
            success: false,
            failure_msg: Some(e.to_string()),
        };
    }

    // kubectl apply -f k8/deployment-canary.yaml
    let output = run_k8_command("kubectl", &["apply", "-f", manifest]).await;

    let success = output.status.success();
    let failure_msg = if success {
        None
    } else {
        Some(String::from_utf8_lossy(&output.stderr).into_owned())
    };

    DeployCanaryResponse {
        success,
        failure_msg,
    }
}

/**
 * 1. Updates the deployment manifest with the new image
 * 2. Rollout the new changes to existing pods
 * 3. Delete the canary deployment
 */
async fn promote_canary(request: PromoteCanaryRequest) -> PromoteCanaryResponse {
    let manifest = "k8/deployment.yaml";
    let canary_manifest = "k8/deployment-canary.yaml";

    if let Err(e) = update_manifest(&request.sha, manifest).await {
        return PromoteCanaryResponse {
            success: false,
            failure_msg: Some(e.to_string()),
        };
    }

    // kubectl apply -f k8/deployment.yaml
    let output = run_k8_command("kubectl", &["apply", "-f", manifest]).await;

    if !output.status.success() {
        return PromoteCanaryResponse {
            success: false,
            failure_msg: Some(String::from_utf8_lossy(&output.stderr).into_owned()),
        };
    }

    suspend_until_rollout_completes("deployment/captain").await;

    let output = run_k8_command("kubectl", &["delete", "-f", canary_manifest]).await;
    let success = output.status.success();
    let failure_msg = if success {
        None
    } else {
        Some(String::from_utf8_lossy(&output.stderr).into_owned())
    };

    PromoteCanaryResponse {
        success,
        failure_msg,
    }
}

async fn update_manifest(image: &str, manifest_path: &str) -> Result<(), ManifestError> {
    let content = tokio::fs::read_to_string(manifest_path).await?;

    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    for line in lines.iter_mut() {
        if line.trim().starts_with("image:") {
            let indent = line.len() - line.trim_start().len();
            let spaces = &line[..indent];
            *line = format!("{}image: {}", spaces, image);
            break;
        }
    }
    let new_content = lines.join("\n");

    tokio::fs::write(manifest_path, new_content).await?;
    Ok(())
}

async fn get_pods() -> GetPodsResponse {
    // kubectl get pods -o json
    let output = run_k8_command("kubectl", &["get", "pods", "-o", "json"]).await;

    if !output.status.success() {
        return GetPodsResponse { pods: vec![] };
    }

    let parsed: serde_json::Value = match serde_json::from_slice(&output.stdout) {
        Ok(val) => val,
        Err(_) => return GetPodsResponse { pods: vec![] },
    };

    let mut pods = Vec::new();

    if let Some(items) = parsed.get("items").and_then(|i| i.as_array()) {
        for item in items {
            let name = item
                .get("metadata")
                .and_then(|m| m.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string();

            let ip_address = item
                .get("status")
                .and_then(|s| s.get("podIP"))
                .and_then(|ip| ip.as_str())
                .unwrap_or("")
                .to_string();

            let is_canary = item
                .get("metadata")
                .and_then(|m| m.get("labels"))
                .and_then(|l| l.get("version"))
                .and_then(|t| t.as_str())
                .map(|t| t == "canary")
                .unwrap_or(false);

            let container_status = item
                .get("status")
                .and_then(|s| s.get("containerStatuses"))
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.first());

            let restart_count = container_status
                .and_then(|cs| cs.get("restartCount"))
                .and_then(|rc| rc.as_u64())
                .unwrap_or(0) as u32;

            let image = container_status
                .and_then(|cs| cs.get("image"))
                .and_then(|img| img.as_str())
                .unwrap_or("")
                .to_string();

            let is_terminating = item
                .get("metadata")
                .and_then(|m| m.get("deletionTimestamp"))
                .is_some();

            let phase = item
                .get("status")
                .and_then(|s| s.get("phase"))
                .and_then(|p| p.as_str())
                .unwrap_or("");

            let status = if is_terminating {
                PodStatus::Terminating
            } else {
                match phase {
                    "Running" => PodStatus::Running,
                    "Pending" => PodStatus::Starting,
                    _ => PodStatus::Terminating,
                }
            };

            let created_at = item
                .get("metadata")
                .and_then(|m| m.get("creationTimestamp"))
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string();

            pods.push(PodInfo {
                name,
                ip_address,
                restart_count,
                is_canary,
                image,
                created_at,
                status,
            });
        }
    }

    GetPodsResponse { pods }
}

async fn wait_for_active_pod_count(expected: usize) {
    for _ in 0..30 {
        if get_active_pod_count().await == expected {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    panic!("Timed out waiting for pod count to reach {}", expected);
}

async fn get_active_pod_count() -> usize {
    let get_pods_response = get_pods().await;
    return get_pods_response
        .pods
        .iter()
        .filter(|x| x.status == PodStatus::Running)
        .count();
}

async fn suspend_until_rollout_completes(deployment_name: &str) {
    run_k8_command("kubectl", &["rollout", "status", deployment_name]).await;
}

async fn run_k8_command(cmd: &str, args: &[&str]) -> Output {
    return Command::new(cmd)
        .args(args)
        .output()
        .await
        .expect("failed to execute process");
}

fn app() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/test", get(test))
}

#[tokio::main]
async fn main() {
    println!("Server running");

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app()).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::Query;
    use axum_test::TestServer;
    use std::collections::HashMap;

    async fn setup() {
        let output = run_k8_command("kubectl", &["apply", "-f", "k8/deployment.yaml"]).await;
        assert!(
            output.status.success(),
            "Failed setup: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        wait_for_active_pod_count(2).await;
    }

    async fn teardown() {
        let output = run_k8_command("kubectl", &["delete", "-f", "k8/deployment.yaml"]).await;
        assert!(output.status.success());
    }

    #[tokio::test]
    async fn test_root() {
        let result = root().await;
        assert_eq!(result, "Hello World");
    }

    #[tokio::test]
    async fn test_test_handler() {
        let mut params = HashMap::new();
        params.insert("param".to_string(), "Rust".to_string());

        let result = test(Query(params)).await;
        assert_eq!(result, "Processed request, Rust!");
    }

    #[tokio::test]
    async fn test_integration_test_server() {
        // Create the TestServer
        let server = TestServer::new(app());

        // Make requests against it
        let response = server.get("/test?param=2").await;

        // Assert response contents
        response.assert_text("Processed request, 2!");
        response.assert_status_ok();
    }

    #[tokio::test]
    #[serial]
    async fn test_scale() {
        setup().await;

        assert_eq!(get_active_pod_count().await, 2);

        let response = scale(ScaleRequest { replica_count: 4 }).await;
        assert!(response.success, "Scale failed: {:?}", response.failure_msg);

        wait_for_active_pod_count(4).await;
        assert_eq!(get_active_pod_count().await, 4);

        teardown().await;
    }

    #[tokio::test]
    #[serial]
    async fn test_restart() {
        setup().await;

        let initial_pods = get_pods().await;
        let initial_names: Vec<String> = initial_pods.pods.into_iter().map(|p| p.name).collect();

        let response = restart().await;
        assert!(
            response.success,
            "Restart failed: {:?}",
            response.failure_msg
        );

        // Don't use wait_for_pod_count(2) because the 2 old pods are still active.
        suspend_until_rollout_completes("deployment/captain").await;

        let current_pods = get_pods().await;
        for pod in current_pods.pods {
            if pod.status == PodStatus::Terminating {
                continue; // Ignore old pods that are still shutting down
            }
            assert!(
                !initial_names.contains(&pod.name),
                "Pod {} was not replaced during restart",
                pod.name
            );
        }

        teardown().await;
    }

    #[tokio::test]
    #[serial]
    async fn canary_test() {
        setup().await;
        let current_image = "docker.io/library/captain:2.0";
        let new_image = "docker.io/library/captain:3.0";
        let _ = update_manifest(current_image, "k8/deployment-canary.yaml").await;

        let pods: Vec<PodInfo> = get_pods()
            .await
            .pods
            .into_iter()
            .filter(|p| p.status == PodStatus::Running)
            .collect();
        for p in pods {
            assert_eq!(p.image, current_image);
        }

        let response = deploy_canary(DeployCanaryRequest {
            sha: new_image.to_string(),
        })
        .await;
        assert!(
            response.success,
            "canary deploy failed: {:?}",
            response.failure_msg
        );

        suspend_until_rollout_completes("deployment/captain-canary").await;

        let canary: PodInfo = get_pods()
            .await
            .pods
            .into_iter()
            .find(|p| p.is_canary)
            .expect("Expected canary pod to be running");

        assert!(canary.name.contains("captain-canary"));
        assert_eq!(canary.image, new_image);

        let response = promote_canary(PromoteCanaryRequest {
            sha: new_image.to_string(),
        })
        .await;
        assert!(
            response.success,
            "promote failed: {:?}",
            response.failure_msg
        );

        suspend_until_rollout_completes("deployment/captain").await;

        let canary: Option<PodInfo> = get_pods()
            .await
            .pods
            .into_iter()
            .filter(|p| p.status == PodStatus::Running)
            .find(|p| p.is_canary);
        assert!(canary.is_none());

        let pods: Vec<PodInfo> = get_pods()
            .await
            .pods
            .into_iter()
            .filter(|p| p.status == PodStatus::Running)
            .collect();
        for p in pods {
            assert_eq!(p.image, new_image);
        }

        let _ = update_manifest(current_image, "k8/deployment.yaml").await;
        teardown().await;
    }
}
