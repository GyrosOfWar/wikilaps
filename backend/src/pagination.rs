use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, Debug, Clone, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PageParameters {
    pub page: Option<u32>,
    pub size: Option<u32>,
    pub sort: Option<String>,
}

impl PageParameters {
    pub const DEFAULT_PAGE: i64 = 1;
    pub const DEFAULT_SIZE: i64 = 20;

    #[allow(unused)]
    pub fn new(page: u32, size: u32) -> Self {
        Self {
            page: Some(page),
            size: Some(size),
            sort: None,
        }
    }

    pub fn limit(&self) -> i64 {
        self.size()
    }

    pub fn offset(&self) -> i64 {
        (self.page() - 1) * self.limit()
    }

    pub fn size(&self) -> i64 {
        self.size.map(|s| s as i64).unwrap_or(Self::DEFAULT_SIZE)
    }

    /// Pages are 1-based; anything lower is clamped so `offset` stays non-negative.
    pub fn page(&self) -> i64 {
        self.page
            .map(|p| p as i64)
            .unwrap_or(Self::DEFAULT_PAGE)
            .max(Self::DEFAULT_PAGE)
    }
}

#[derive(Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    pub content: Vec<T>,
    pub total_items: u32,
    pub page_number: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

impl<T> Page<T> {
    pub fn empty() -> Self {
        Page {
            content: vec![],
            total_items: 0,
            page_number: PageParameters::DEFAULT_PAGE as u32,
            page_size: PageParameters::DEFAULT_SIZE as u32,
            total_pages: 0,
        }
    }
}

impl<T> Page<T> {
    pub fn new(content: Vec<T>, total_size: u32, page: PageParameters) -> Self {
        let page_number = page.page() as u32;
        let page_size = page.size() as u32;
        let total_pages = (total_size as f64 / page_size as f64).ceil() as u32;

        Page {
            content,
            total_items: total_size,
            page_number,
            page_size,
            total_pages,
        }
    }

    pub fn map<F, U>(self, fun: F) -> Page<U>
    where
        F: FnMut(T) -> U,
    {
        Page {
            content: self.content.into_iter().map(fun).collect(),
            total_items: self.total_items,
            page_number: self.page_number,
            page_size: self.page_size,
            total_pages: self.total_pages,
        }
    }
}

#[cfg(test)]
mod test {
    use super::PageParameters;

    #[test]
    fn first_page_starts_at_offset_zero() {
        assert_eq!(PageParameters::new(1, 20).offset(), 0);
        assert_eq!(PageParameters::new(2, 20).offset(), 20);
        assert_eq!(PageParameters::new(3, 15).offset(), 30);
    }

    #[test]
    fn defaults_to_the_first_page() {
        let params = PageParameters {
            page: None,
            size: None,
            sort: None,
        };
        assert_eq!(params.page(), 1);
        assert_eq!(params.offset(), 0);
    }

    #[test]
    fn clamps_pages_below_one() {
        assert_eq!(PageParameters::new(0, 20).page(), 1);
        assert_eq!(PageParameters::new(0, 20).offset(), 0);
    }
}
