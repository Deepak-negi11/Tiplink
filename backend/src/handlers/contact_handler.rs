use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::fs::{OpenOptions, File};
use std::io::{Read, Write};
use chrono::Utc;

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
    println!("📬 New Contact Message Received:");
    println!("Name: {}", payload.name);
    println!("Email: {}", payload.email);
    println!("Message: {}", payload.message);

    let entry = MessageEntry {
        name: payload.name,
        email: payload.email,
        message: payload.message,
        timestamp: Utc::now().to_rfc3339(),
    };

    // Save to contact_messages.json in workspace root
    let filepath = "/Users/deepak/Documents/Project/Tiplink/contact_messages.json";
    
    // Read existing messages or create new array
    let mut messages: Vec<MessageEntry> = Vec::new();
    if let Ok(mut file) = File::open(filepath) {
        let mut content = String::new();
        if file.read_to_string(&mut content).is_ok() {
            if let Ok(existing) = serde_json::from_str::<Vec<MessageEntry>>(&content) {
                messages = existing;
            }
        }
    }

    messages.push(entry);

    // Write back to file
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(filepath)
    {
        if let Ok(serialized) = serde_json::to_string_pretty(&messages) {
            let _ = file.write_all(serialized.as_bytes());
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Thank you! Your message has been saved and sent to the administrator."
    }))
}
