use actix_web::{HttpResponse, Responder, web};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize, Debug)]
pub struct ContactPayload {
    pub name: String,
    pub email: String,
    pub message: String,
}

pub async fn submit_message(payload: web::Json<ContactPayload>) -> impl Responder {
    let payload = payload.into_inner();
    let name = payload.name.trim();
    let email = payload.email.trim();
    let message = payload.message.trim();

    if name.is_empty() || email.is_empty() || message.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "message": "Name, email, and message are required."
        }));
    }

    if !email.contains('@') {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "message": "Enter a valid email address."
        }));
    }

    let api_key = std::env::var("RESEND_API_KEY").unwrap_or_default();
    if api_key.trim().is_empty() {
        return HttpResponse::ServiceUnavailable().json(json!({
            "success": false,
            "message": "Email delivery is not configured yet. Please email deepaknegi108r@gmail.com directly."
        }));
    }

    let from = std::env::var("RESEND_FROM")
        .unwrap_or_else(|_| "Orbit Contact <onboarding@resend.dev>".to_string());
    let to = std::env::var("CONTACT_TO_EMAIL")
        .unwrap_or_else(|_| "deepaknegi108r@gmail.com".to_string());
    let email_body = format!("Name: {name}\nEmail: {email}\n\nMessage:\n{message}");

    let response = Client::new()
        .post("https://api.resend.com/emails")
        .bearer_auth(api_key)
        .json(&json!({
            "from": from,
            "to": [to],
            "reply_to": email,
            "subject": format!("Orbit enquiry from {name}"),
            "text": email_body
        }))
        .send()
        .await;

    match response {
        Ok(response) if response.status().is_success() => HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Your message has been sent."
        })),
        Ok(response) => {
            let status = response.status();
            let error = response.text().await.unwrap_or_default();
            eprintln!("Resend API error ({status}): {error}");
            HttpResponse::BadGateway().json(json!({
                "success": false,
                "message": "Your message could not be sent. Please email deepaknegi108r@gmail.com directly."
            }))
        }
        Err(error) => {
            eprintln!("Failed to contact Resend: {error}");
            HttpResponse::BadGateway().json(json!({
                "success": false,
                "message": "Your message could not be sent. Please email deepaknegi108r@gmail.com directly."
            }))
        }
    }
}
