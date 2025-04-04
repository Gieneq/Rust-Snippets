use std::sync::{Arc, Mutex};

use serde::{
    Deserialize, 
    Serialize
};

use crate::game::{
    common::Vector2F, 
    world::{Entity, EntityId, World}
};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientRequest {
    GetId,
    WorldCheck,
    Healthcheck,
}

#[derive(Serialize, Deserialize)]
pub struct EntityCheckData {
    position: Vector2F,
    id: EntityId,
    name: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientResponse {
    GetId {
        id: EntityId
    },
    WorldCheck {
        entities: Vec<EntityCheckData>
    },
    Healthcheck {
        msg: String
    },
    BadRequest {
        err: String
    },
    OtherError {
        err: String
    },
}



impl EntityCheckData {
    fn vec_from_iter<'a, I: Iterator<Item = &'a Entity>>(iter: I) -> Vec<Self> {
        iter.map(|e| {
            EntityCheckData {
                name: e.name.clone(),
                id: e.id,
                position: e.position
            }
        })
        .collect()
    }
}

pub fn route_request(player_id: EntityId, request_str: &str, world: Arc<Mutex<World>>) -> String {
    let response: ClientResponse = match serde_json::from_str::<ClientRequest>(request_str) {
        Ok(req) => match req {
            ClientRequest::GetId => {
                ClientResponse::GetId { id: player_id }
            },
            ClientRequest::WorldCheck => {
                match world.lock() {
                    Ok(world_guard) => {
                        ClientResponse::WorldCheck { 
                            entities: EntityCheckData::vec_from_iter(world_guard.iter_entities())
                        }
                    },
                    Err(e) => {
                        ClientResponse::OtherError { err: e.to_string() }
                    }
                }
            },
            ClientRequest::Healthcheck => {
                
                match world.lock() {
                    Ok(world_guard) => {
                        let players_count = world_guard.iter_entities().filter(|e| e.is_player()).count();
                        ClientResponse::Healthcheck { msg: format!("Hello from server! Players active {players_count}.") }
                    },
                    Err(e) => {
                        ClientResponse::OtherError { err: e.to_string() }
                    }
                }
            },
        },
        Err(e) => ClientResponse::BadRequest { err: format!("request={request_str}, reason={e}") },
    };

    serde_json::to_string(&response).expect("Could not serialize response")
}