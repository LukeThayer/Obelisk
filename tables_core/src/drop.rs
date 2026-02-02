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
