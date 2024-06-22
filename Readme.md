# Substruct

## Description
This is a small proc-macro used to remove the specified fields from a struct. 
This is useful when you have relatively similar structs whose difference is that one has more fields than the other. 
For example a request to create a customer might take the shape 
```json
{ "name" : "John Smith", "address" : "New York", "currency" : "usd"}
```

while the response has 
```
{ "id" : "83937220", "name" : "John Smith", "address" : "New York", "currency" : "usd"}
```

In such a case, one would define the struct with all the fields. 

```rust
struct Customer {
    id : String,
    name : String,
    address : String,
    currency : String}
```

and then use this library to create a new struct `CreateCustomer` as a subset of `Customer` without the `id` field

```
    [substruct(Customer, CreateCustomer ["id"])]
```
