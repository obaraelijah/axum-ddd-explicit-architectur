use domain::{
    aggregate::{circle::Circle, value_object::circle_id::CircleId},
    interface::circle_repository_interface::CircleRepositoryInterface,
};
use super::db_data::{circle_data::CircleData, member_data::MemberData};
use sqlx::Row;

#[derive(Clone, Debug)]
pub struct CircleRepositoryWithMySql {
    db: sqlx::MySqlPool,
}

impl CircleRepositoryWithMySql {
    pub fn new(db: sqlx::MySqlPool) -> Self {
        Self { db }
    }
}

impl CircleRepositoryInterface for CircleRepositoryWithMySql {
    async fn find_by_id(&self, circle_id: &CircleId) -> Result<Circle, anyhow::Error> {
        tracing::info!("find_circle_by_id : {:?}", circle_id);
        let circle_query =
            sqlx::query("SELECT * FROM circles WHERE id = ?").bind(circle_id.to_string());

        let circle_row = circle_query.fetch_one(&self.db).await.map_err(|e| {
            eprintln!("Failed to fetch circle by id: {:?}", e);
            anyhow::Error::msg("Failed to fetch circle by id")
        })?;

        let member_query =
            sqlx::query("SELECT * FROM members WHERE circle_id = ?").bind(circle_id.to_string());

        let members_row = member_query.fetch_all(&self.db).await.map_err(|e| {
            eprintln!("Failed to fetch members by circle id: {:?}", e);
            anyhow::Error::msg("Failed to fetch members by circle id")
        })?;

        let members: Vec<MemberData> = members_row
            .into_iter()
            .map(|member| MemberData {
                id: member.get::<i16, _>("id"),
                name: member.get::<String, _>("name"),
                age: member.get::<i16, _>("age"),
                grade: member.get::<i16, _>("grade"),
                major: member.get::<String, _>("major"),
            })
            .collect();

        let owner: MemberData = members
            .iter()
            .find(|member| member.id == circle_row.get::<i16, _>("owner_id"))
            .ok_or_else(|| anyhow::Error::msg("Owner not found"))?
            .clone();

        let circle_data = CircleData {
            id: circle_row.get::<i16, _>("id"),
            name: circle_row.get::<String, _>("name"),
            owner_id: circle_row.get::<i16, _>("owner_id"),
            owner,
            capacity: circle_row.get::<i16, _>("capacity"),
            members,
        };

        Ok(Circle::try_from(circle_data)?)
    }

}