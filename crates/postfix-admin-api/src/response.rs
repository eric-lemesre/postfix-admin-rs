//! Standardized API response wrappers.
//!
//! All API endpoints return responses wrapped in `{"data": ...}` for single
//! entities or `{"data": [...], "meta": {...}}` for paginated lists.

use serde::Serialize;
use utoipa::ToSchema;

use postfix_admin_core::pagination::PageResponse;

/// Single-entity response: `{"data": T}`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ApiResponse<T: Serialize> {
    pub data: T,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

impl<T: Serialize> From<T> for ApiResponse<T> {
    fn from(data: T) -> Self {
        Self::new(data)
    }
}

/// Pagination metadata included in list responses.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PaginationMeta {
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

/// Paginated list response: `{"data": [...], "meta": {...}}`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ApiListResponse<T: Serialize> {
    pub data: Vec<T>,
    pub meta: PaginationMeta,
}

impl<T: Serialize> ApiListResponse<T> {
    /// Wrap a non-paginated `Vec<T>` as a single-page response.
    #[must_use]
    pub fn from_vec(items: Vec<T>) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        let total = items.len() as u64;
        #[allow(clippy::cast_possible_truncation)]
        let per_page = items.len() as u32;
        Self {
            data: items,
            meta: PaginationMeta {
                total,
                page: 1,
                per_page: if per_page == 0 { 25 } else { per_page },
                total_pages: u32::from(total != 0),
            },
        }
    }
}

impl<T: Serialize> From<PageResponse<T>> for ApiListResponse<T> {
    fn from(page: PageResponse<T>) -> Self {
        let total_pages = page.total_pages();
        Self {
            meta: PaginationMeta {
                total: page.total,
                page: page.page,
                per_page: page.per_page,
                total_pages,
            },
            data: page.items,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_response_serializes_with_data_key() {
        let resp = ApiResponse::new("hello");
        let json = serde_json::to_value(&resp).unwrap_or_else(|_| unreachable!());
        assert_eq!(json["data"], "hello");
        assert!(json.get("meta").is_none());
    }

    #[test]
    fn api_response_from_struct() {
        #[derive(Serialize)]
        struct Item {
            id: u32,
        }
        let resp = ApiResponse::new(Item { id: 42 });
        let json = serde_json::to_value(&resp).unwrap_or_else(|_| unreachable!());
        assert_eq!(json["data"]["id"], 42);
    }

    #[test]
    fn api_list_response_serializes_with_data_and_meta() {
        let resp = ApiListResponse::from_vec(vec![1, 2, 3]);
        let json = serde_json::to_value(&resp).unwrap_or_else(|_| unreachable!());
        assert_eq!(json["data"], serde_json::json!([1, 2, 3]));
        assert_eq!(json["meta"]["total"], 3);
        assert_eq!(json["meta"]["page"], 1);
        assert_eq!(json["meta"]["per_page"], 3);
        assert_eq!(json["meta"]["total_pages"], 1);
    }

    #[test]
    fn api_list_response_from_page_response() {
        let page = PageResponse {
            items: vec!["a", "b"],
            total: 50,
            page: 2,
            per_page: 25,
        };
        let resp: ApiListResponse<&str> = page.into();
        let json = serde_json::to_value(&resp).unwrap_or_else(|_| unreachable!());
        assert_eq!(json["data"], serde_json::json!(["a", "b"]));
        assert_eq!(json["meta"]["total"], 50);
        assert_eq!(json["meta"]["page"], 2);
        assert_eq!(json["meta"]["per_page"], 25);
        assert_eq!(json["meta"]["total_pages"], 2);
    }

    #[test]
    fn api_list_response_empty_vec() {
        let resp = ApiListResponse::<String>::from_vec(vec![]);
        let json = serde_json::to_value(&resp).unwrap_or_else(|_| unreachable!());
        assert_eq!(json["data"], serde_json::json!([]));
        assert_eq!(json["meta"]["total"], 0);
        assert_eq!(json["meta"]["total_pages"], 0);
    }
}
