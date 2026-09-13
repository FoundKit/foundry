use serde::{Deserialize, Serialize};

/// Standard successful response envelope in Foundry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: u32,
    pub message: String,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            message: "success".to_string(),
            data,
            meta: None,
        }
    }

    pub fn with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            code: 0,
            message: message.into(),
            data,
            meta: None,
        }
    }

    pub fn with_meta(data: T, meta: serde_json::Value) -> Self {
        Self {
            code: 0,
            message: "success".to_string(),
            data,
            meta: Some(meta),
        }
    }
}

/// Paginated data metadata envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMeta {
    pub page: u64,
    pub page_size: u64,
    pub total: u64,
    pub total_pages: u64,
}

impl PageMeta {
    pub fn new(page: u64, page_size: u64, total: u64) -> Self {
        let total_pages = if page_size == 0 {
            0
        } else {
            total.div_ceil(page_size)
        };
        Self {
            page,
            page_size,
            total,
            total_pages,
        }
    }
}

/// Paginated response data wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedData<T> {
    pub items: Vec<T>,
    pub pagination: PageMeta,
}

impl<T> PaginatedData<T> {
    pub fn new(items: Vec<T>, page: u64, page_size: u64, total: u64) -> Self {
        Self {
            items,
            pagination: PageMeta::new(page, page_size, total),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response() {
        let res = ApiResponse::success("hello");
        assert_eq!(res.code, 0);
        assert_eq!(res.message, "success");
        assert_eq!(res.data, "hello");
        assert!(res.meta.is_none());

        let res2 = ApiResponse::with_message("data", "created");
        assert_eq!(res2.message, "created");

        let res3 = ApiResponse::with_meta("data", serde_json::json!({ "took_ms": 12 }));
        assert!(res3.meta.is_some());
    }

    #[test]
    fn test_page_meta_calculations() {
        let p1 = PageMeta::new(1, 20, 100);
        assert_eq!(p1.total_pages, 5);

        let p2 = PageMeta::new(1, 20, 101);
        assert_eq!(p2.total_pages, 6);

        let p3 = PageMeta::new(1, 20, 0);
        assert_eq!(p3.total_pages, 0);

        let p4 = PageMeta::new(1, 0, 100);
        assert_eq!(p4.total_pages, 0);

        let paginated = PaginatedData::new(vec![1, 2, 3], 1, 10, 3);
        assert_eq!(paginated.items.len(), 3);
        assert_eq!(paginated.pagination.total, 3);
    }
}
