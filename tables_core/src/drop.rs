/// Result of rolling a drop table
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Drop {
    /// An item to generate with optional currencies to apply
    Item {
        base_type: String,
        currencies: Vec<String>,
    },
    /// A currency drop with a count
    Currency { id: String, count: u32 },
    /// A unique item to generate
    Unique { id: String },
}

/// An item drop extracted from a Drop list
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemDrop<'a> {
    pub base_type: &'a str,
    pub currencies: &'a [String],
}

/// A currency drop extracted from a Drop list
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrencyDrop<'a> {
    pub id: &'a str,
    pub count: u32,
}

/// A unique drop extracted from a Drop list
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniqueDrop<'a> {
    pub id: &'a str,
}

/// Extension trait for working with collections of Drops
pub trait DropsExt {
    /// Get all item drops
    fn get_items(&self) -> Vec<ItemDrop<'_>>;

    /// Get all currency drops
    fn get_currencies(&self) -> Vec<CurrencyDrop<'_>>;

    /// Get all unique drops
    fn get_uniques(&self) -> Vec<UniqueDrop<'_>>;
}

impl DropsExt for [Drop] {
    fn get_items(&self) -> Vec<ItemDrop<'_>> {
        self.iter()
            .filter_map(|d| match d {
                Drop::Item {
                    base_type,
                    currencies,
                } => Some(ItemDrop {
                    base_type,
                    currencies,
                }),
                _ => None,
            })
            .collect()
    }

    fn get_currencies(&self) -> Vec<CurrencyDrop<'_>> {
        self.iter()
            .filter_map(|d| match d {
                Drop::Currency { id, count } => Some(CurrencyDrop { id, count: *count }),
                _ => None,
            })
            .collect()
    }

    fn get_uniques(&self) -> Vec<UniqueDrop<'_>> {
        self.iter()
            .filter_map(|d| match d {
                Drop::Unique { id } => Some(UniqueDrop { id }),
                _ => None,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_items() {
        let drops = vec![
            Drop::Item {
                base_type: "sword".into(),
                currencies: vec!["transmute".into()],
            },
            Drop::Currency {
                id: "gold".into(),
                count: 10,
            },
            Drop::Item {
                base_type: "shield".into(),
                currencies: vec![],
            },
        ];

        let items = drops.get_items();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].base_type, "sword");
        assert_eq!(items[1].base_type, "shield");
    }

    #[test]
    fn test_get_currencies() {
        let drops = vec![
            Drop::Item {
                base_type: "sword".into(),
                currencies: vec![],
            },
            Drop::Currency {
                id: "gold".into(),
                count: 10,
            },
            Drop::Currency {
                id: "chaos".into(),
                count: 2,
            },
        ];

        let currencies = drops.get_currencies();
        assert_eq!(currencies.len(), 2);
        assert_eq!(currencies[0].id, "gold");
        assert_eq!(currencies[0].count, 10);
        assert_eq!(currencies[1].id, "chaos");
    }

    #[test]
    fn test_get_uniques() {
        let drops = vec![
            Drop::Unique {
                id: "starforge".into(),
            },
            Drop::Currency {
                id: "gold".into(),
                count: 5,
            },
            Drop::Unique {
                id: "headhunter".into(),
            },
        ];

        let uniques = drops.get_uniques();
        assert_eq!(uniques.len(), 2);
        assert_eq!(uniques[0].id, "starforge");
        assert_eq!(uniques[1].id, "headhunter");
    }
}
