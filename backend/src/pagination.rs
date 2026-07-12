use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PageParameters {
    pub page: Option<u32>,
    pub size: Option<u32>,
    pub sort: Option<String>,
    pub dir: Option<SortDirection>,
}

impl PageParameters {
    pub const DEFAULT_PAGE: i64 = 0;
    pub const DEFAULT_SIZE: i64 = 20;

    #[allow(unused)]
    pub fn new(page: u32, size: u32) -> Self {
        Self {
            page: Some(page),
            size: Some(size),
            sort: None,
            dir: None,
        }
    }

    pub fn limit(&self) -> i64 {
        self.size.map(|s| s as i64).unwrap_or(Self::DEFAULT_SIZE)
    }

    pub fn offset(&self) -> i64 {
        self.page
            .map(|p| p as i64 * self.limit())
            .unwrap_or(Self::DEFAULT_PAGE)
    }

    pub fn size(&self) -> i64 {
        self.size.map(|s| s as i64).unwrap_or(Self::DEFAULT_SIZE)
    }

    pub fn page(&self) -> i64 {
        self.page.map(|p| p as i64).unwrap_or(Self::DEFAULT_PAGE)
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
            page_number: 0,
            page_size: 0,
            total_pages: 0,
        }
    }
}

impl<T> Page<T> {
    pub fn new(content: Vec<T>, total_size: u32, page: PageParameters) -> Self {
        let page_number = page.page.unwrap_or(PageParameters::DEFAULT_PAGE as u32);
        let page_size = page.size.unwrap_or(PageParameters::DEFAULT_SIZE as u32);
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
