use std::{collections::HashMap, error::Error};
use chrono::NaiveDate;
use serde::{Deserialize, Deserializer};

// Record Struct

fn deserialize_isbn<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    Ok(opt.map(|s| {
        if s.starts_with("=\"") && s.ends_with('"') {
            s[2..s.len() - 1].to_string()
        } else {
            s
        }
    }))
}

#[derive(Deserialize, Clone)]
pub struct Record {
    #[serde(rename = "Book Id")]
    pub book_id: Option<String>,
    #[serde(rename = "Title")]
    pub title: Option<String>,
    #[serde(rename = "Author")]
    pub author: Option<String>,
    #[serde(rename = "Author l-f")]
    pub author_lf: Option<String>,
    #[serde(rename = "Additional Authors")]
    pub additional_authors: Option<String>,
    #[serde(rename = "ISBN", deserialize_with = "deserialize_isbn")]
    pub isbn: Option<String>,
    #[serde(rename = "ISBN13", deserialize_with = "deserialize_isbn")]
    pub isbn_13: Option<String>,
    #[serde(rename = "My Rating")]
    pub rating: Option<i32>,
    #[serde(rename = "Publisher")]
    pub publisher: Option<String>,
    #[serde(rename = "Binding")]
    pub binding: Option<String>,
    #[serde(rename = "Number of Pages")]
    pub number_of_pages: Option<i64>,
    #[serde(rename = "Year Published")]
    pub year_published: Option<i64>,
    #[serde(rename = "Original Publication Year")]
    pub original_publication_year: Option<i64>,
    #[serde(rename = "Date Read")]
    pub date_read: Option<String>,
    #[serde(rename = "Date Added")]
    pub date_added: Option<String>,
    #[serde(rename = "Bookshelves")]
    pub bookshelves: Option<String>,
    #[serde(rename = "Bookshelves with positions")]
    pub bookeshelves_with_positions: Option<String>,
    #[serde(rename = "Exclusive Shelf")]
    pub exclusive_shelf: Option<String>,
    #[serde(rename = "My Review")]
    pub my_review: Option<String>,
    #[serde(rename = "Spoiler")]
    pub spoiler: Option<String>,
    #[serde(rename = "Private Notes")]
    pub private_notes: Option<String>,
    #[serde(rename = "Read Count")]
    pub read_count: Option<i32>,
    #[serde(rename = "Owned Copies")]
    pub owned_copies: Option<i32>
}

// Section Struct

pub struct SectionStats {
    pub rating_section: RatingStats,
    pub page_section: PageStats,
    pub author_section: AuthorStats,
    pub speed_section: SpeedStats    
}

struct SectionCollector {
    rating_collector: RatingCollector,
    page_collector: PageCollector,
    author_collector: AuthorCollector,
    speed_collector: SpeedCollector
}

impl SectionCollector {
    fn new() -> Self {
        Self {
            rating_collector: RatingCollector::default(),
            page_collector: PageCollector::default(),
            author_collector: AuthorCollector::default(),
            speed_collector: SpeedCollector::default()
        }
    }

    fn update(&mut self, record: &Record) {
        self.rating_collector.update(record);
        self.page_collector.update(record);
        self.author_collector.update(record);
        self.speed_collector.update(record);
    }

    fn finalize(self) -> SectionStats {
        SectionStats {
            rating_section: self.rating_collector.finalize(),
            page_section: self.page_collector.finalize(),
            author_section: self.author_collector.finalize(),
            speed_section: self.speed_collector.finalize()
        }
    }
}

// Rating Struct

pub struct RatingStats {
    pub average_rating: f32,
    pub top_rating: i32,
    pub low_rating: i32,
    pub rating_breakdown: Vec<(i32, i32)>
}

#[derive(Default)]
struct RatingCollector {
    sum: i32,
    count: i32,
    top: i32,
    low: i32,
    breakdown: HashMap<i32, i32>
}

impl RatingCollector {
    fn update(&mut self, record: &Record) {
        let Some(rating) = record.rating else {
            return;
        };

        if rating > 0 {
            self.sum += rating;
            self.count += 1;
            if self.count == 1 || rating > self.top { self.top = rating; }
            if self.count == 1 || rating < self.low { self.low = rating; }

            *self.breakdown.entry(rating).or_insert(0) += 1;
        }
    }

    fn finalize(self) -> RatingStats {
        let average_rating = if self.count > 0 { self.sum as f32 / self.count as f32 } else { 0.0 };
        let mut rating_breakdown: Vec<(i32, i32)> = self.breakdown.into_iter().collect();
        rating_breakdown.sort_by_key(|k| k.0);

        RatingStats {
            average_rating,
            top_rating: if self.count > 0 { self.top } else { 0 },
            low_rating: if self.count > 0 { self.low } else { 0 },
            rating_breakdown
        }
    }
}

// Page Struct

pub struct PageStats {
    pub total_pages: i64,
    pub longest_pages: i64,
    pub shortest_pages: i64,
    pub average_pages: f64,
    pub total_books: i32
}

#[derive(Default)]
struct PageCollector {
    sum_pages: i64,
    count_books: i32,
    longest: i64,
    shortest: i64
}

impl PageCollector {
    fn update(&mut self, record: &Record) {
        let Some(pages) = record.number_of_pages else {
            return;
        };

        if pages > 0 {
            self.sum_pages += pages;
            self.count_books += 1;

            if self.count_books == 1 || pages > self.longest { self.longest = pages; }
            if self.count_books == 1 || pages < self.shortest { self.shortest = pages; }
        }
    }

    fn finalize(self) -> PageStats {
        let average_pages = if self.count_books > 0 { self.sum_pages as f64 / self.count_books as f64 } else { 0.0 };

        PageStats {
            total_pages: self.sum_pages,
            longest_pages: if self.count_books > 0 { self.longest } else { 0 },
            shortest_pages: if self.count_books > 0 { self.shortest } else { 0 },
            average_pages,
            total_books: self.count_books,
        }
    }
}

// Author Struct

pub struct AuthorStats {
    pub author_breakdown: Vec<(String, i32)>,
    pub total_authors: i32
}

#[derive(Default)]
struct AuthorCollector {
    breakdown: HashMap<String, i32>
}

impl AuthorCollector {
    fn update(&mut self, record: &Record) {
        if let Some(author) = &record.author {
            *self.breakdown.entry(author.clone()).or_insert(0) += 1;
        }
    }

    fn finalize(self) -> AuthorStats {
        let total_authors = self.breakdown.len() as i32;
        let mut author_breakdown: Vec<(String, i32)> = self.breakdown.into_iter().collect();
        author_breakdown.sort_by(|a, b| b.1.cmp(&a.1));
        
        AuthorStats {
            author_breakdown,
            total_authors
        }
    }
}

// Speed Struct

pub struct SpeedStats {
    pub months_breakdown: Vec<(String, i32)>,
    pub average_speed: f64
}

#[derive(Default)]
struct SpeedCollector {
    speed: i64,
    count: i32,
    breakdown: HashMap<String, i32>
}

impl SpeedCollector {
    fn update(&mut self, record: &Record) {
        let (Some(read_str), Some(added_str)) = (&record.date_read, &record.date_added) else {
            return;
        };

        let parse_format = "%Y/%m/%d";
        let read_date = match NaiveDate::parse_from_str(read_str, parse_format) {
            Ok(date) => date,
            Err(_) => return
        };

        let added_date = match NaiveDate::parse_from_str(added_str, parse_format) {
            Ok(date) => date,
            Err(_) => return
        };

        let duration = read_date.signed_duration_since(added_date);
        let days = duration.num_days();

        if days >= 0 {
            self.speed += days;
            self.count += 1;

            let month_key = read_date.format("%B").to_string();
            *self.breakdown.entry(month_key).or_insert(0) += 1;
        }
    }

    fn finalize(self) -> SpeedStats {
        let average_speed = if self.count > 0 { self.speed as f64 / self.count as f64 } else { 0.0 };
        let mut months_breakdown: Vec<(String, i32)> = self.breakdown.into_iter().collect();
        months_breakdown.sort_by(|a, b| b.1.cmp(&a.1));

        SpeedStats {
            months_breakdown,
            average_speed
        }
    }
}

pub fn process_records(records: Vec<Record>) -> Result<SectionStats, Box<dyn Error>> {
    let mut collector = SectionCollector::new();

    for record in &records {
        collector.update(record);
    }

    Ok(collector.finalize())
}
