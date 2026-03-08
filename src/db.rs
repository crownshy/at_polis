use sea_orm::{ConnectionTrait, Database, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::models::{Poll, PollRef, Statement, StatementRef, Vote, VoteValue};

// Polls entity
pub mod poll_entity {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "polls")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub uri: String,
        pub cid: String,
        pub did: String,
        pub topic: String,
        pub description: Option<String>,
        pub created_at: DateTimeUtc,
        pub closed_at: Option<DateTimeUtc>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

// Statements entity
pub mod statement_entity {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "statements")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub uri: String,
        pub cid: String,
        pub did: String,
        pub text: String,
        pub poll_uri: String,
        pub poll_cid: String,
        pub created_at: DateTimeUtc,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

// Votes entity
pub mod vote_entity {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "votes")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub uri: String,
        pub cid: String,
        pub did: String,
        pub value: String, // "agree", "disagree", "pass"
        pub statement_uri: String,
        pub statement_cid: String,
        pub poll_uri: String,
        pub poll_cid: String,
        pub created_at: DateTimeUtc,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

/// Initialize the database and create tables
pub async fn init_db(database_url: &str) -> Result<DatabaseConnection, sea_orm::DbErr> {
    let db = Database::connect(database_url).await?;

    // Create tables using raw SQL
    db.execute_unprepared(
        r#"
        CREATE TABLE IF NOT EXISTS polls (
            uri TEXT PRIMARY KEY NOT NULL,
            cid TEXT NOT NULL,
            did TEXT NOT NULL,
            topic TEXT NOT NULL,
            description TEXT,
            created_at TEXT NOT NULL,
            closed_at TEXT
        )
        "#,
    )
    .await?;

    db.execute_unprepared(
        r#"
        CREATE TABLE IF NOT EXISTS statements (
            uri TEXT PRIMARY KEY NOT NULL,
            cid TEXT NOT NULL,
            did TEXT NOT NULL,
            text TEXT NOT NULL,
            poll_uri TEXT NOT NULL,
            poll_cid TEXT NOT NULL,
            created_at TEXT NOT NULL
        )
        "#,
    )
    .await?;

    db.execute_unprepared(
        r#"
        CREATE TABLE IF NOT EXISTS votes (
            uri TEXT PRIMARY KEY NOT NULL,
            cid TEXT NOT NULL,
            did TEXT NOT NULL,
            value TEXT NOT NULL,
            statement_uri TEXT NOT NULL,
            statement_cid TEXT NOT NULL,
            poll_uri TEXT NOT NULL,
            poll_cid TEXT NOT NULL,
            created_at TEXT NOT NULL
        )
        "#,
    )
    .await?;

    // Create indexes
    db.execute_unprepared("CREATE INDEX IF NOT EXISTS idx_statements_poll ON statements(poll_uri)")
        .await
        .ok();

    db.execute_unprepared("CREATE INDEX IF NOT EXISTS idx_votes_statement ON votes(statement_uri)")
        .await
        .ok();

    db.execute_unprepared("CREATE INDEX IF NOT EXISTS idx_votes_poll ON votes(poll_uri)")
        .await
        .ok();

    Ok(db)
}

/// Insert or update a poll
pub async fn upsert_poll(
    db: &DatabaseConnection,
    uri: &str,
    cid: &str,
    did: &str,
    poll: &Poll,
) -> Result<(), sea_orm::DbErr> {
    let model = poll_entity::ActiveModel {
        uri: Set(uri.to_string()),
        cid: Set(cid.to_string()),
        did: Set(did.to_string()),
        topic: Set(poll.topic.clone()),
        description: Set(poll.description.clone()),
        created_at: Set(poll.created_at),
        closed_at: Set(poll.closed_at),
    };

    poll_entity::Entity::insert(model)
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(poll_entity::Column::Uri)
                .update_columns([
                    poll_entity::Column::Cid,
                    poll_entity::Column::Topic,
                    poll_entity::Column::Description,
                    poll_entity::Column::ClosedAt,
                ])
                .to_owned(),
        )
        .exec(db)
        .await?;

    Ok(())
}

/// Insert or update a statement
pub async fn upsert_statement(
    db: &DatabaseConnection,
    uri: &str,
    cid: &str,
    did: &str,
    statement: &Statement,
) -> Result<(), sea_orm::DbErr> {
    let model = statement_entity::ActiveModel {
        uri: Set(uri.to_string()),
        cid: Set(cid.to_string()),
        did: Set(did.to_string()),
        text: Set(statement.text.clone()),
        poll_uri: Set(statement.poll.uri.clone()),
        poll_cid: Set(statement.poll.cid.clone()),
        created_at: Set(statement.created_at),
    };

    statement_entity::Entity::insert(model)
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(statement_entity::Column::Uri)
                .update_columns([
                    statement_entity::Column::Cid,
                    statement_entity::Column::Text,
                ])
                .to_owned(),
        )
        .exec(db)
        .await?;

    Ok(())
}

/// Insert or update a vote
pub async fn upsert_vote(
    db: &DatabaseConnection,
    uri: &str,
    cid: &str,
    did: &str,
    vote: &Vote,
) -> Result<(), sea_orm::DbErr> {
    let vote_value_str = match vote.value {
        VoteValue::Agree => "agree",
        VoteValue::Disagree => "disagree",
        VoteValue::Pass => "pass",
    };

    let model = vote_entity::ActiveModel {
        uri: Set(uri.to_string()),
        cid: Set(cid.to_string()),
        did: Set(did.to_string()),
        value: Set(vote_value_str.to_string()),
        statement_uri: Set(vote.subject.uri.clone()),
        statement_cid: Set(vote.subject.cid.clone()),
        poll_uri: Set(vote.poll.uri.clone()),
        poll_cid: Set(vote.poll.cid.clone()),
        created_at: Set(vote.created_at),
    };

    vote_entity::Entity::insert(model)
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(vote_entity::Column::Uri)
                .update_columns([vote_entity::Column::Cid, vote_entity::Column::Value])
                .to_owned(),
        )
        .exec(db)
        .await?;

    Ok(())
}

/// Get all polls
pub async fn get_polls(db: &DatabaseConnection) -> Result<Vec<Poll>, sea_orm::DbErr> {
    use sea_orm::QueryOrder;

    let models = poll_entity::Entity::find()
        .order_by_desc(poll_entity::Column::CreatedAt)
        .all(db)
        .await?;

    let results: Vec<Poll> = models
        .into_iter()
        .map(|model| Poll {
            topic: model.topic,
            description: model.description,
            created_at: model.created_at,
            closed_at: model.closed_at,
            did: model.did,
            cid: model.cid,
            uri: model.uri,
        })
        .collect();

    Ok(results)
}

/// Returns the next statement that the current user has yet to
/// vote on
pub async fn next_statements_for_user_on_poll(
    db: &DatabaseConnection,
    poll_uri: &str,
    user_uri: &str,
) -> Result<Option<Statement>, anyhow::Error> {
    let models = statement_entity::Entity::find()
        .filter(statement_entity::Column::PollUri.eq(poll_uri))
        .order_by_asc(statement_entity::Column::CreatedAt)
        .all(db)
        .await?;
}

/// Get all statements for a poll
pub async fn get_statements_for_poll(
    db: &DatabaseConnection,
    poll_uri: &str,
) -> Result<Vec<Statement>, sea_orm::DbErr> {
    use sea_orm::{ColumnTrait, QueryFilter, QueryOrder};

    let models = statement_entity::Entity::find()
        .filter(statement_entity::Column::PollUri.eq(poll_uri))
        .order_by_asc(statement_entity::Column::CreatedAt)
        .all(db)
        .await?;

    let results = models
        .into_iter()
        .map(|model| Statement {
            text: model.text,
            poll: PollRef {
                uri: model.poll_uri,
                cid: model.poll_cid,
            },
            created_at: model.created_at,
            did: model.did,
            cid: model.cid,
            uri: model.uri,
        })
        .collect();

    Ok(results)
}

/// Get all votes for a statement
pub async fn get_votes_for_statement(
    db: &DatabaseConnection,
    statement_uri: &str,
) -> Result<Vec<Vote>, sea_orm::DbErr> {
    use sea_orm::{ColumnTrait, QueryFilter, QueryOrder};

    let models = vote_entity::Entity::find()
        .filter(vote_entity::Column::StatementUri.eq(statement_uri))
        .order_by_asc(vote_entity::Column::CreatedAt)
        .all(db)
        .await?;

    let results = models
        .into_iter()
        .map(|model| {
            let value = match model.value.as_str() {
                "agree" => VoteValue::Agree,
                "disagree" => VoteValue::Disagree,
                "pass" => VoteValue::Pass,
                _ => VoteValue::Pass,
            };

            Vote {
                value,
                subject: StatementRef {
                    uri: model.statement_uri,
                    cid: model.statement_cid,
                },
                poll: PollRef {
                    uri: model.poll_uri,
                    cid: model.poll_cid,
                },
                did: model.did,
                cid: model.cid,
                uri: model.uri,
                created_at: model.created_at,
            }
        })
        .collect();

    Ok(results)
}
