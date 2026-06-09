use actix_web::{HttpResponse, Responder, web};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug)]
pub struct ContactPayload {
    pub name: String,
    pub email: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageEntry {
    pub name: String,
    pub email: String,
    pub message: String,
    pub timestamp: String,
}

pub async fn submit_message(payload: web::Json<ContactPayload>) -> impl Responder {
    let payload = payload.into_inner();
    if payload.name.trim().is_empty()
        || payload.email.trim().is_empty()
        || payload.message.trim().is_empty()
    {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": "Name, email, and message are required."
        }));
    }

    let entry = MessageEntry {
        name: payload.name.trim().to_string(),
        email: payload.email.trim().to_string(),
        message: payload.message.trim().to_string(),
        timestamp: Utc::now().to_rfc3339(),
    };

    let filepath = std::env::var("CONTACT_MESSAGES_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("contact_messages.json"));

    let mut messages: Vec<MessageEntry> = Vec::new();
    if let Ok(mut file) = File::open(&filepath) {
        let mut content = String::new();
        if file.read_to_string(&mut content).is_ok() {
            if let Ok(existing) = serde_json::from_str::<Vec<MessageEntry>>(&content) {
                messages = existing;
            }
        }
    }

    messages.push(entry);

    let serialized = match serde_json::to_string_pretty(&messages) {
        Ok(serialized) => serialized,
        Err(error) => {
            eprintln!("Failed to serialize contact message: {error}");
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "Unable to save your message right now. Please email deepaknegi108r@gmail.com."
            }));
        }
    };

    match OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&filepath)
    {
        Ok(mut file) => {
            if let Err(error) = file.write_all(serialized.as_bytes()) {
                eprintln!("Failed to write contact message: {error}");
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "message": "Unable to save your message right now. Please email deepaknegi108r@gmail.com."
                }));
            }
        }
        Err(error) => {
            eprintln!("Failed to open contact message file: {error}");
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "Unable to save your message right now. Please email deepaknegi108r@gmail.com."
            }));
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Thank you! Your message has been received."
    }))
}
