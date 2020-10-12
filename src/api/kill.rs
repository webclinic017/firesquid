use hyper::{Body, Request, Response, StatusCode};
use tracing::error;

use super::{build_response, VmInput};
use crate::state::StatePtr;

//TODO: process kill into vm package
pub async fn handler(
    request: Request<Body>,
    state_ptr: StatePtr,
) -> Result<Response<Body>, hyper::Error> {
    let body_bytes = &hyper::body::to_bytes(request.into_body()).await?;

    let body: VmInput = match serde_json::from_slice(body_bytes) {
        Ok(j) => j,
        Err(e) => {
            error!("{}", e);

            let response = build_response(StatusCode::OK, e.to_string());
            return Ok(response);
        }
    };

    let mut pid: u32 = 0;

    {
        let state = state_ptr.lock().await;
        if let Some(vm) = state.vms.iter().find(|vm| vm.name == body.vm_name) {
            pid = vm.pid;
        };
    }

    if pid == 0 {
        let response = build_response(
            StatusCode::OK,
            format!("Machine not found: {}", body.vm_name),
        );
        return Ok(response);
    }

    let child = match tokio::process::Command::new("kill")
        .arg(pid.to_string())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            let response = build_response(StatusCode::OK, format!("Error killing vm: {}", e));
            return Ok(response);
        }
    };

    if let Err(e) = child.await {
        let response = build_response(StatusCode::OK, format!("Error killing vm: {}", e));
        return Ok(response);
    };

    let response = build_response(
        StatusCode::OK,
        serde_json::json!({
            "sucess": true,
        })
        .to_string(),
    );
    Ok(response)
}
