use tera::{Context, Tera};

use crate::deployment_status;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        match Tera::new("templates/*.html") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        }
    };
}

pub async fn load_index() -> anyhow::Result<String> {
    let mut context = Context::new();
    let deployments = deployment_status::list().await?;
    context.insert("deployments", &deployments);

    Ok(TEMPLATES.render("index.html", &context)?)
}
