use super::schema::*;

#[derive(Queryable)]
pub struct Factoid {
    pub id: i32,
    pub key: String,
    pub pred: String,
    pub value: String,
}

#[derive(Insertable)]
#[table_name = "brain"]
pub struct NewFactoid {
    #[column_name = "fact_key"]
    pub key: String,
    #[column_name = "fact_pred"]
    pub pred: String,
    #[column_name = "fact_val"]
    pub value: String,
}
