use serde::Deserialize;
use crate::{indexes::Index, errors::Error, request::*};
use std::collections::{BTreeMap, HashSet, BTreeSet};
use serde_json::{Value, from_value};

#[derive(Deserialize)]
pub(crate) struct ProgressJson {
    pub(crate) updateId: usize,
}

impl ProgressJson {
    pub(crate) fn into_progress<'a>(self, index: &'a Index) -> Progress<'a> {
        Progress {
            id: self.updateId,
            index
        }
    }
}

/// A struct used to track the progress of some async operations.
pub struct Progress<'a> {
    id: usize,
    index: &'a Index<'a>,
}

impl<'a> Progress<'a> {
    ///
    /// # Example
    /// 
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, documents::*};
    /// let client = Client::new("http://localhost:7700", "");
    /// let mut movies_index = client.get_or_create("movies").unwrap();
    /// let progress = movies_index.delete_all_documents().unwrap();
    /// let status = progress.get_status().unwrap();
    /// ```
    pub fn get_status(&self) -> Result<Status, Error> {
        let value = request::<(), serde_json::Value>(
            &format!("{}/indexes/{}/updates/{}", self.index.client.host, self.index.uid, self.id),
            self.index.client.apikey,
            Method::Get,
            200,
        )?;
        if let Ok(status) = from_value::<ProcessedStatus>(value.clone()) {
            return Ok(Status::Processed(status));
        } else if let Ok(status) = from_value::<EnqueuedStatus>(value) {
            return Ok(Status::Enqueued(status));
        }
        Err(Error::Unknown("Invalid server response, src/progress.rs:49:9".to_string()))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum RankingRule {
    Typo,
    Words,
    Proximity,
    Attribute,
    WordsPosition,
    Exactness,
    Asc(String),
    Dsc(String),
}

#[derive(Debug, Clone, Deserialize)]
pub enum UpdateState<T> {
    Update(T),
    Clear,
    Nothing,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsUpdate {
    pub ranking_rules: UpdateState<Vec<RankingRule>>,
    pub distinct_attribute: UpdateState<String>,
    pub identifier: UpdateState<String>,
    pub searchable_attributes: UpdateState<Vec<String>>,
    pub displayed_attributes: UpdateState<HashSet<String>>,
    pub stop_words: UpdateState<BTreeSet<String>>,
    pub synonyms: UpdateState<BTreeMap<String, Vec<String>>>,
    pub accept_new_fields: UpdateState<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "name")]
pub enum UpdateType {
    ClearAll,
    Customs,
    DocumentsAddition { number: usize },
    DocumentsPartial { number: usize },
    DocumentsDeletion { number: usize },
    Settings { settings: SettingsUpdate },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessedStatus {
    pub update_id: u64,
    #[serde(rename = "type")]
    pub update_type: UpdateType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub duration: f64, // in seconds
    pub enqueued_at: String, // TODO deserialize to datatime
    pub processed_at: String, // TODO deserialize to datatime
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnqueuedStatus {
    pub update_id: u64,
    #[serde(rename = "type")]
    pub update_type: UpdateType,
    pub enqueued_at: String, // TODO deserialize to datatime
}


pub enum Status {
    Processed(ProcessedStatus),
    Enqueued(EnqueuedStatus),
}
