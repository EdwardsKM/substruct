use substruct::sub_struct;

#[sub_struct(name = "CreateAddressParams", remove = ["id", "created"])]
#[derive(Debug, PartialEq)]
struct Address {
    id: String,
    country: String,
    state: Option<String>,
    city: String,
    street: String,
    line_1: String,
    line_2: String,
    created: i64,
}
