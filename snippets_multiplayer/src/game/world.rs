use super::common::Vector2F;

#[derive(Debug)]
pub struct World {
    new_entity_id: EntityId,
    entities: Vec<Entity>,
}

#[derive(Debug, PartialEq)]
pub enum EntityState {
    Idle,
    Moving,
}

type EntityId = u32;

#[derive(Debug)]
pub struct Entity {
    id: u32,
    pub name: String,
    pub position: Vector2F,
    state: EntityState,
}

impl World {
    pub fn new() -> Self {
        println!("World created");
        Self {
            new_entity_id: 0,
            entities: vec![],
        }
    }

    pub fn create_entity<S: AsRef<str>>(&mut self, name: S, intial_position: Vector2F) -> EntityId {
        let new_id = self.new_entity_id;
        self.new_entity_id += 1;

        let entity = Entity { 
            id: new_id, 
            name: name.as_ref().to_string(),
            position: intial_position,
            state: EntityState::Idle,
        };

        self.entities.push(entity);
        new_id
    }

    pub fn get_entity_by_id(&self, entity_id: EntityId) -> Option<&Entity> {
        self.entities.iter().find(|e| e.id == entity_id)
    }

    pub fn get_entity_by_id_mut(&mut self, entity_id: EntityId) -> Option<&mut Entity> {
        self.entities.iter_mut().find(|e| e.id == entity_id)
    }

    pub fn tick(&mut self) {
        println!("World tick");
    }
}

#[test]
fn test_world_creation() {
    let world = World::new();
    assert_eq!(world.new_entity_id, 0);
}

#[test]
fn test_world_entity_creation_should_increase_entities_count() {
    let mut world = World::new();
    assert_eq!(world.new_entity_id, 0);

    let new_entity_id = world.create_entity("Bob", Vector2F::new(1.0, 2.0));
    assert_eq!(new_entity_id, 0);

    assert_eq!(world.new_entity_id, 1);
}

#[test]
fn test_world_entity_access() {
    let entity_name = "Bob";
    let entity_position = Vector2F::new(1.0, 2.0);

    let mut world = World::new();
    let new_entity_id = world.create_entity(entity_name, entity_position);

    let entity = world.get_entity_by_id(new_entity_id).unwrap();
    assert_eq!(entity.name, entity_name);
    assert_eq!(entity.position, entity_position);
    assert_eq!(entity.state, EntityState::Idle);
}

#[test]
fn test_world_entity_translate() {
    let entity_initial_position = Vector2F::new(1.0, 2.0);
    let translation = Vector2F::new(100.0, 500.0);

    let mut world = World::new();
    let new_entity_id = world.create_entity("Bob", entity_initial_position);

    let entity = world.get_entity_by_id_mut(new_entity_id).unwrap();
    entity.position += translation;
    assert_eq!(entity.position, entity_initial_position + translation);
}