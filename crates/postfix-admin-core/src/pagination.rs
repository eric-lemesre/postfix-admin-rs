use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

const DEFAULT_PAGE_SIZE: u32 = 25;
const MAX_PAGE_SIZE: u32 = 100;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum SortDirection {
    #[default]
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

#[derive(Debug, Clone, Serialize, Deserialize, IntoParams)]
pub struct PageRequest {
    page: u32,
    per_page: u32,
    sort_by: Option<String>,
    sort_direction: SortDirection,
}

impl PageRequest {
    #[must_use]
    pub fn new(page: u32, per_page: u32) -> Self {
        Self {
            page: page.max(1),
            per_page: per_page.clamp(1, MAX_PAGE_SIZE),
            sort_by: None,
            sort_direction: SortDirection::default(),
        }
    }

    #[must_use]
    pub fn with_sort(mut self, field: impl Into<String>, direction: SortDirection) -> Self {
        self.sort_by = Some(field.into());
        self.sort_direction = direction;
        self
    }

    #[must_use]
    pub fn page(&self) -> u32 {
        self.page
    }

    #[must_use]
    pub fn per_page(&self) -> u32 {
        self.per_page
    }

    #[must_use]
    pub fn offset(&self) -> u64 {
        u64::from(self.page - 1) * u64::from(self.per_page)
    }

    #[must_use]
    pub fn sort_by(&self) -> Option<&str> {
        self.sort_by.as_deref()
    }

    #[must_use]
    pub fn sort_direction(&self) -> SortDirection {
        self.sort_direction
    }
}

impl Default for PageRequest {
    fn default() -> Self {
        Self::new(1, DEFAULT_PAGE_SIZE)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
}

impl<T> PageResponse<T> {
    #[must_use]
    pub fn new(items: Vec<T>, total: u64, request: &PageRequest) -> Self {
        Self {
            items,
            total,
            page: request.page(),
            per_page: request.per_page(),
        }
    }

    #[must_use]
    pub fn total_pages(&self) -> u32 {
        if self.total == 0 {
            return 0;
        }
        #[allow(clippy::cast_possible_truncation)]
        {
            self.total.div_ceil(u64::from(self.per_page)) as u32
        }
    }

    #[must_use]
    pub fn has_next(&self) -> bool {
        self.page < self.total_pages()
    }

    #[must_use]
    pub fn has_prev(&self) -> bool {
        self.page > 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_request_offset_first_page() {
        let req = PageRequest::new(1, 25);
        assert_eq!(req.offset(), 0);
    }

    #[test]
    fn page_request_offset_second_page() {
        let req = PageRequest::new(2, 25);
        assert_eq!(req.offset(), 25);
    }

    #[test]
    fn page_request_clamps_per_page_to_max() {
        let req = PageRequest::new(1, 200);
        assert_eq!(req.per_page(), MAX_PAGE_SIZE);
    }

    #[test]
    fn page_request_clamps_page_to_min() {
        let req = PageRequest::new(0, 25);
        assert_eq!(req.page(), 1);
    }

    #[test]
    fn page_response_total_pages_exact_division() {
        let resp: PageResponse<()> = PageResponse {
            items: vec![],
            total: 100,
            page: 1,
            per_page: 25,
        };
        assert_eq!(resp.total_pages(), 4);
    }

    #[test]
    fn page_response_total_pages_with_remainder() {
        let resp: PageResponse<()> = PageResponse {
            items: vec![],
            total: 101,
            page: 1,
            per_page: 25,
        };
        assert_eq!(resp.total_pages(), 5);
    }

    #[test]
    fn page_response_total_pages_zero_total() {
        let resp: PageResponse<()> = PageResponse {
            items: vec![],
            total: 0,
            page: 1,
            per_page: 25,
        };
        assert_eq!(resp.total_pages(), 0);
    }

    #[test]
    fn page_response_has_next_true() {
        let resp: PageResponse<()> = PageResponse {
            items: vec![],
            total: 50,
            page: 1,
            per_page: 25,
        };
        assert!(resp.has_next());
    }

    #[test]
    fn page_response_has_next_false_on_last_page() {
        let resp: PageResponse<()> = PageResponse {
            items: vec![],
            total: 50,
            page: 2,
            per_page: 25,
        };
        assert!(!resp.has_next());
    }
}
