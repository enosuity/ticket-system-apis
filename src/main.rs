use std::{fmt::Display, sync::Mutex};

use actix_web::{
    body::BoxBody, delete, get, http::{header::ContentType, StatusCode}, post, put, web, App, HttpRequest, HttpResponse, HttpServer, Responder, ResponseError
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Ticket {
    id: u8,
    name: String,
}

impl Responder for Ticket {
    type Body = BoxBody;
    
    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        let res_body = serde_json::to_string(&self).unwrap();

        HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(res_body)        
    }    
}
#[derive(Debug, Serialize)]
struct ErrNoId {
    id: u8,
    err: String,
}

impl ResponseError for ErrNoId {
    fn status_code(&self) -> StatusCode {
        StatusCode::NOT_FOUND
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let body = serde_json::to_string(&self).unwrap();
        let res = HttpResponse::new(self.status_code());
        res.set_body(BoxBody::new(body))
    }    
}

impl Display for ErrNoId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

struct AppState {
    tickets: Mutex<Vec<Ticket>>,
}

// Get all Tickets.
#[get("/tickets")]
async fn all_tickets(data: web::Data<AppState>) -> impl Responder {
    let tickets = data.tickets.lock().unwrap();

    let res = serde_json::to_string(&(*tickets)).unwrap();

    HttpResponse::Ok()
    .content_type(ContentType::json())
    .body(res)
}

#[post("/tickets")]
async fn create_ticket(req: web::Json<Ticket>, data: web::Data<AppState>) -> impl Responder {
    let new_ticket = Ticket {
        id: req.id,
        name: String::from(&req.name),
    };
    let mut tickets = data.tickets.lock().unwrap();
    let res = serde_json::to_string(&new_ticket).unwrap();

    tickets.push(new_ticket);


    HttpResponse::Ok()
    .content_type(ContentType::json())
    .body(res)
}

#[get("tickets/{id}")]
async fn get_ticket(id: web::Path<u8>, data: web::Data<AppState>) -> Result<Ticket, ErrNoId> {
    let ticket_id: u8 = *id;
    let tickets = data.tickets.lock().unwrap();

    let ticket: Vec<_> = tickets.iter()
                                .filter(|x| x.id == ticket_id)
                                .collect();
    if !ticket.is_empty() {
        Ok(
            Ticket {
                id: ticket[0].id,
                name: String::from(&ticket[0].name),
            }
        )
    } else {
        let res = ErrNoId {
            id: ticket_id,
            err: String::from("ticket not found")
        };
        Err(res)
    }
}

#[put("tickets/{id}")]
async fn update_ticket(id: web::Path<u8>, req: web::Json<Ticket>, data: web::Data<AppState>) -> Result<HttpResponse, ErrNoId> {
    let ticket_id: u8 = *id;

    if ticket_id != req.id {
        let res = ErrNoId {
            id: ticket_id,
            err: String::from("ticket id did not match")
        };
        return Err(res);
    }
    let mut tickets = data.tickets.lock().unwrap();

    let pos = tickets.iter().position(|x| x.id == ticket_id);
    let new_ticket = Ticket {
        id: req.id,
        name: String::from(&req.name),
    };

    match pos {
        Some(index) => {
            let res = serde_json::to_string(&new_ticket).unwrap();

            tickets[index] = new_ticket;
            Ok(
                HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(res)
            )
        },
        None => {
            let res = ErrNoId {
                id: ticket_id,
                err: String::from("ticket not found")
            };
            Err(res) 
        }
        
    }
}

#[delete("tickets/{id}")]
async fn remove_ticket(id: web::Path<u8>, data: web::Data<AppState>) -> Result<Ticket, ErrNoId> {
    let ticket_id: u8 = *id;
    let mut tickets = data.tickets.lock().unwrap();

    let pos = tickets.iter().position(|x| x.id == ticket_id);
    match pos {
        Some(index) => {
            let deleted_ticket = tickets.remove(index);
            Ok(deleted_ticket)
        },
        None => {
            let res = ErrNoId {
                id: ticket_id,
                err: String::from("ticket not found")
            };
            Err(res) 
        }        
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
   let app_state = web::Data::new(
    AppState {
       tickets: Mutex::new(vec![
            Ticket {
                id: 1,
                name: "Anuj".to_string(),
            },
            Ticket {
                id: 2,
                name: "Goldy".to_string(),
            }
        ]) 
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(all_tickets)
            .service(get_ticket)
            .service(create_ticket)
            .service(update_ticket)
            .service(remove_ticket)
    }) 
    .bind(("127.0.0.1", 8000))?
    .run()
    .await


}
