// use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::{apps::v1::Deployment, core::v1::Pod};
use kube::{
    api::{Api, ListParams, ResourceExt},
    Client,
};

fn label_selector(deployment: Deployment) -> Option<String> {
    let labels = deployment
        .spec?
        .selector
        .match_labels?
        .iter()
        .map(|(key, value)| format!("{}={}", key, value))
        .collect::<Vec<_>>()
        .join(",");
    Some(labels)
}

fn pod_status_message(pod: &Pod) -> Option<String> {
    let messages = pod
        .status
        .as_ref()?
        .container_statuses
        .as_ref()?
        .into_iter()
        .filter_map(|status| status.state.as_ref()?.waiting.as_ref()?.message.clone())
        .collect::<Vec<_>>()
        .join("\n");
    Some(messages)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Infer the runtime environment and try to create a Kubernetes Client
    let client = Client::try_default().await?;

    let deployment: Api<Deployment> = Api::all(client.clone());
    for d in deployment
        .list(&ListParams::default())
        .await?
        .into_iter()
        .filter(|d| d.metadata.name.and_then(|name| name.starts_with("kube-").))
    {
        let name = dbg!(d.name_any());
        let status = d.status.as_ref().unwrap();
        let ready_replicas = status.ready_replicas.unwrap_or_default();
        let replicas = status.replicas.unwrap_or_default();

        let status_message = if ready_replicas != replicas {
            let pods: Api<Pod> = Api::default_namespaced(client.clone());
            let Some(label_selector) = label_selector(d) else {
                continue;
            };
            let lp = ListParams::default().labels(&label_selector);
            let mut messages: Vec<String> = pods
                .list(&lp)
                .await
                .unwrap()
                .iter()
                .filter_map(|pod| pod_status_message(pod))
                .collect();
            messages.dedup();
            messages
        } else {
            Vec::new()
        };
        println!(
            "{name} {ready_replicas}/{replicas} {}",
            status_message.join("\n")
        );
    }

    Ok(())
}
