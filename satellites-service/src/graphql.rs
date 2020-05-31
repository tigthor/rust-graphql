use std::str::FromStr;

use async_graphql::*;
use chrono::prelude::*;
use strum_macros::EnumString;

use crate::persistence::connection::PgPool;
use crate::persistence::model::SatelliteEntity;
use crate::persistence::repository;

pub type AppSchema = Schema<Query, EmptyMutation, EmptySubscription>;

pub struct Query;

#[Object(extends)]
impl Query {
    async fn satellites(&self, ctx: &Context<'_>) -> Vec<Satellite> {
        let conn = ctx.data::<PgPool>().get().expect("Can't get DB connection");

        let satellite_entities = repository::all(&conn).expect("Can't get satellites");

        satellite_entities.iter()
            .map(|e| { convert(e) })
            .collect()
    }

    async fn satellite(&self, ctx: &Context<'_>, id: ID) -> Option<Satellite> {
        let conn = ctx.data::<PgPool>().get().expect("Can't get DB connection");

        let id = id.to_string().parse::<i32>().expect("Can't get id from String");
        repository::get(id, &conn).ok()
            .map(|e| { convert(&e) })
    }

    #[entity]
    async fn find_planet_by_id(&self, ctx: &Context<'_>, id: ID) -> Planet {
        let satellites_stub: Vec<Satellite> = vec![];
        Planet {
            id: id.clone(),
            satellites: satellites_stub,
        }
    }
}

#[SimpleObject]
#[derive(Clone)]
struct Satellite {
    id: ID,
    name: String,
    life_exists: LifeExists,
    first_spacecraft_landing_date: Option<NaiveDate>,
    planet_id: i32,
}

#[Enum]
#[derive(EnumString)]
enum LifeExists {
    Yes,
    OpenQuestion,
    NoData,
}

#[derive(Clone)]
struct Planet {
    id: ID,
    satellites: Vec<Satellite>,
}

#[Object(extends)]
impl Planet {
    #[field(external)]
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn satellites(&self, ctx: &Context<'_>) -> Vec<Satellite> {
        let conn = ctx.data::<PgPool>().get().expect("Can't get DB connection");

        let id = self.id.to_string().parse::<i32>().expect("Can't get id from String");
        let satellite_entities = repository::get_by_planet_id(id, &conn).expect("Can't get satellites of planet");

        let satellites = satellite_entities.iter()
            .map(|e| { convert(e) })
            .collect();

        satellites
    }
}

// todo from/into trait
fn convert(satellite_entity: &SatelliteEntity) -> Satellite {
    Satellite {
        id: satellite_entity.id.into(),
        name: satellite_entity.name.clone(),
        life_exists: LifeExists::from_str(satellite_entity.life_exists.as_str()).expect("Can't convert &str to LifeExists"),
        first_spacecraft_landing_date: satellite_entity.first_spacecraft_landing_date,
        planet_id: satellite_entity.planet_id,
    }
}
