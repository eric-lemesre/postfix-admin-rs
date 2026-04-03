use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use postfix_admin_core::dto::{CreateLog, LogFilter, LogResponse};
use postfix_admin_core::error::CoreError;
use postfix_admin_core::pagination::{PageRequest, PageResponse};
use postfix_admin_core::repository::LogRepository;

use crate::rows::{CountRow, LogRow};

pub struct PgLogRepository {
    pool: PgPool,
}

impl PgLogRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LogRepository for PgLogRepository {
    async fn create(&self, dto: &CreateLog) -> Result<LogResponse, CoreError> {
        let id = Uuid::now_v7();
        let row = sqlx::query_as::<_, LogRow>(
            "INSERT INTO log (id, username, domain, action, data, ip_address, user_agent) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             RETURNING id, timestamp, username, domain, action, data, \
             ip_address, user_agent",
        )
        .bind(id)
        .bind(&dto.username)
        .bind(&dto.domain)
        .bind(&dto.action)
        .bind(dto.data.as_deref().unwrap_or(""))
        .bind(dto.ip_address.as_deref())
        .bind(dto.user_agent.as_deref())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::repository(e.to_string()))?;

        Ok(row.into())
    }

    async fn find_all(
        &self,
        filter: &LogFilter,
        page: &PageRequest,
    ) -> Result<PageResponse<LogResponse>, CoreError> {
        let mut where_clauses = Vec::new();
        let mut param_idx = 1u32;

        if filter.domain.is_some() {
            where_clauses.push(format!("domain = ${param_idx}"));
            param_idx += 1;
        }
        if filter.username.is_some() {
            where_clauses.push(format!("username = ${param_idx}"));
            param_idx += 1;
        }
        if filter.action.is_some() {
            where_clauses.push(format!("action = ${param_idx}"));
            param_idx += 1;
        }
        if filter.from.is_some() {
            where_clauses.push(format!("timestamp >= ${param_idx}"));
            param_idx += 1;
        }
        if filter.until.is_some() {
            where_clauses.push(format!("timestamp <= ${param_idx}"));
            param_idx += 1;
        }

        let where_str = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let count_sql = format!("SELECT COUNT(*) AS count FROM log {where_str}");
        let mut count_query = sqlx::query_as::<_, CountRow>(&count_sql);

        if let Some(ref d) = filter.domain {
            count_query = count_query.bind(d);
        }
        if let Some(ref u) = filter.username {
            count_query = count_query.bind(u);
        }
        if let Some(ref a) = filter.action {
            count_query = count_query.bind(a);
        }
        if let Some(ref f) = filter.from {
            count_query = count_query.bind(f);
        }
        if let Some(ref u) = filter.until {
            count_query = count_query.bind(u);
        }

        let total = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        let select_sql = format!(
            "SELECT id, timestamp, username, domain, action, data, ip_address, user_agent \
             FROM log {where_str} ORDER BY timestamp DESC LIMIT ${param_idx} OFFSET ${}",
            param_idx + 1
        );
        let mut select_query = sqlx::query_as::<_, LogRow>(&select_sql);

        if let Some(ref d) = filter.domain {
            select_query = select_query.bind(d);
        }
        if let Some(ref u) = filter.username {
            select_query = select_query.bind(u);
        }
        if let Some(ref a) = filter.action {
            select_query = select_query.bind(a);
        }
        if let Some(ref f) = filter.from {
            select_query = select_query.bind(f);
        }
        if let Some(ref u) = filter.until {
            select_query = select_query.bind(u);
        }

        #[allow(clippy::cast_possible_wrap)]
        let offset = page.offset() as i64;
        select_query = select_query.bind(i64::from(page.per_page())).bind(offset);

        let rows = select_query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CoreError::repository(e.to_string()))?;

        let items = rows.into_iter().map(Into::into).collect();
        #[allow(clippy::cast_sign_loss)]
        Ok(PageResponse::new(items, total.count as u64, page))
    }
}
