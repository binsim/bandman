#[cfg(test)]
use crate::models::WishlistCategory;
use crate::models::{SongCategory, WishlistItem};

#[derive(Clone, PartialEq, Eq)]
pub(super) enum WishlistDisplayEntry {
    Song(Box<WishlistItem>),
    Medley { id: uuid::Uuid },
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct WishlistCategoryGroup {
    pub(super) category: Option<SongCategory>,
    pub(super) entries: Vec<WishlistDisplayEntry>,
}

impl WishlistCategoryGroup {
    pub(super) fn key(&self) -> String {
        self.category
            .as_ref()
            .map(|category| category.id.to_string())
            .unwrap_or_else(|| "uncategorized".to_string())
    }
}

impl WishlistDisplayEntry {
    pub(super) fn key(&self) -> String {
        match self {
            Self::Song(item) => format!("song-{}", item.id),
            Self::Medley { id } => format!("medley-{id}"),
        }
    }
}

pub(super) fn group_wishlist_items_by_category(
    items: Vec<WishlistItem>,
) -> Vec<WishlistCategoryGroup> {
    let medley_categories: std::collections::HashMap<_, _> = items
        .iter()
        .filter_map(|item| {
            item.medley_id
                .map(|medley_id| (medley_id, item.medley_category.clone()))
        })
        .collect();
    let mut groups: Vec<WishlistCategoryGroup> = Vec::new();

    for entry in group_wishlist_items(items) {
        let category = match &entry {
            WishlistDisplayEntry::Song(item) => item.song_category.clone(),
            WishlistDisplayEntry::Medley { id } => medley_categories.get(id).cloned().flatten(),
        };
        let group = groups.iter_mut().find(|group| {
            group.category.as_ref().map(|item| item.id) == category.as_ref().map(|item| item.id)
        });
        if let Some(group) = group {
            group.entries.push(entry);
        } else {
            groups.push(WishlistCategoryGroup {
                category,
                entries: vec![entry],
            });
        }
    }
    groups.sort_by(|left, right| match (&left.category, &right.category) {
        (Some(left), Some(right)) => left
            .name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
            .then_with(|| left.name.cmp(&right.name)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });
    groups
}

pub(super) fn group_wishlist_items(items: Vec<WishlistItem>) -> Vec<WishlistDisplayEntry> {
    let mut entries = Vec::new();
    let mut medley_ids = std::collections::HashSet::new();
    for item in items {
        let Some(medley_id) = item.medley_id else {
            entries.push(WishlistDisplayEntry::Song(Box::new(item)));
            continue;
        };
        if medley_ids.insert(medley_id) {
            entries.push(WishlistDisplayEntry::Medley { id: medley_id });
        }
    }
    entries
}

pub(super) fn sort_wishlist_items(items: &mut [WishlistItem]) {
    let mut group_titles = std::collections::HashMap::new();
    for item in items.iter() {
        if let Some(medley_id) = item.medley_id {
            group_titles.entry(medley_id).or_insert_with(|| {
                item.medley_name
                    .clone()
                    .unwrap_or_else(|| item.title.clone())
            });
        }
    }
    items.sort_by(|left, right| {
        let left_group_title = left
            .medley_id
            .and_then(|id| group_titles.get(&id))
            .unwrap_or(&left.title);
        let right_group_title = right
            .medley_id
            .and_then(|id| group_titles.get(&id))
            .unwrap_or(&right.title);
        left_group_title
            .to_lowercase()
            .cmp(&right_group_title.to_lowercase())
            .then_with(|| match (left.medley_id, right.medley_id) {
                (Some(left_id), Some(right_id)) if left_id == right_id => {
                    left.medley_position.cmp(&right.medley_position)
                }
                (Some(left_id), Some(right_id)) => left_id.cmp(&right_id),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            })
            .then_with(|| left.title.cmp(&right.title))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(
        title: &str,
        medley_id: Option<uuid::Uuid>,
        song_category: Option<SongCategory>,
        medley_category: Option<SongCategory>,
    ) -> WishlistItem {
        WishlistItem {
            id: uuid::Uuid::new_v4(),
            proposed_by_id: None,
            medley_id,
            medley_position: None,
            medley_size: medley_id.map(|_| 2),
            medley_created_by_id: None,
            medley_name: medley_id.map(|_| "Medley".to_string()),
            title: title.to_string(),
            artist: None,
            link: None,
            link_title: None,
            tempo: None,
            musical_key: None,
            targets: Vec::new(),
            category: WishlistCategory::Song,
            song_category,
            labels: Vec::new(),
            medley_category,
            medley_labels: Vec::new(),
            feedback: Vec::new(),
            proposed_by: String::new(),
            created_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn category_view_keeps_uncategorized_songs() {
        let groups =
            group_wishlist_items_by_category(vec![item("Uncategorized", None, None, None)]);

        assert_eq!(groups.len(), 1);
        assert!(groups[0].category.is_none());
        assert_eq!(groups[0].entries.len(), 1);
    }

    #[test]
    fn category_view_treats_medley_as_one_item() {
        let category = SongCategory {
            id: uuid::Uuid::new_v4(),
            name: "Party".into(),
        };
        let medley_id = uuid::Uuid::new_v4();
        let groups = group_wishlist_items_by_category(vec![
            item("First song", Some(medley_id), None, Some(category.clone())),
            item("Second song", Some(medley_id), None, Some(category.clone())),
        ]);

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].category, Some(category));
        assert!(matches!(
            groups[0].entries.as_slice(),
            [WishlistDisplayEntry::Medley { .. }]
        ));
    }
}
