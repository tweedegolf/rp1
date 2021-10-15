# The `crud` macro
The crud macro is the main entrypoint for RP1.

## Example
```rust
#[rp1::crud(database = "Db", table = "users", auth = false)]
struct User {
    #[primary_key]
    pub id: i32,
    #[validate(email)]
    username: String,
    role: String,
    #[generated]
    created_at: rp1::datetime::OffsetDateTime,
    #[generated]
    updated_at: rp1::datetime::OffsetDateTime,
}
```

## Macro properties
The following properties can be specified on the main crud attribute macro.

* `database: Path`: Path to the Rocket database struct. This is the struct
  for which you added the `#[database(...)]` attribute from
  `rocket_sync_db_pools`. By default we assume such a struct is called `Db`.
  Note that this value must be provided in string quotes because of parser
  limitations.
* `table: Ident`: Name of the table in the diesel schema on which queries
  should be executed. Take a look at your `schema.rs` file or take a look in
  your database. By default this is assumed to be the snake case variant of
  your struct name.
* `schema: Path`: The path to your schema definition. By default we assume that
  this path is `crate::schema`, but if you changed the path for the schema or
  if the schema is located in another crate you should specify this property.
  Note that this value must be provided as a string because of parser
  limitations.
* `module: Ident`: The name of the module where the generated code should go.
  By default we use a name based on the name of the struct and convert it to
  snake case (so `UserPreferrences` whould become `user_preferrences`).
* `max_limit: i64`: The maximum number of items returned in a listview. By
  default this is set to 100.
* `auth: bool`: Whether or not to enable authorization, take a look at the
  authorization section for more details on this. By default authorization is
  enabled.
* `create: bool`: Whether or not to enable the create endpoint, by default this
  is enabled.
* `read: bool`: Whether or not to enable the read endpoint, by default this is
  enabled.
* `update: bool`: Whether or not to enable the update endpoint, by default this
  is enabled.
* `delete: bool`: Whether or not to enable the delete endpoint, by default this
  is enabled.
* `list: bool`: Whether or not to enable the list endpoint, by default this is
  enabled.
* `partials: bool`: Whether or not to enable support for partial results, by
  default this is enabled, but it can be disabled for a slight performance
  boost.

## Field attributes
There are several field attributes you can add to a field in your struct to
indicate some special meaning for that field, these are:

| Attribute           | Description                                           |
|---------------------|-------------------------------------------------------|
| `#[generated]`      | Generated fields that cannot be inserted/updated.     |
| `#[primary_key]`    | The primary key that will be used as an id.           |
| `#[not_sortable]`   | Indicates that a field cannot be used to sort.        |
| `#[not_filterable]` | Indicates that a field cannot be used for filtering.  |

## Authorization
RP1 allows you to modify the behavior of your endpoints based on some auth
object. This auth object can be anything that implements the rocket
`FromRequest` trait (see the guards section in the rocket documentation). You
should make sure that your FromRequest guard makes sure that the user struct or
enum you retrieve is valid and authenticated. Then, implement the
`CheckPermissions` trait. For more details on the CheckPermissions trait, check
its documentation.

## Generated API endpoints
Once you added the macro to some struct, you should mount the generated routes
in your rocket application. To do this, add a call to `mount` to your
`rocket::build` call. The mountable routes can be retrieved by calling the
`get_routes` function on your struct. The full function call should be
something like `.mount("/users", User::get_routes())`.

In the sections below all urls are relative to this mounted path. So for
example when `GET /1` is used below, we actually mean `GET /users/1`.

In general all your requests should include an `Accept: application/json` to
ensure that all responses will be JSON formatted. If you don't, you may end up
getting a HTML response from Rocket.

### Create: `POST /`
Send a post request on the root route to create a new entity. The post body
should never include a generated or primary key column. The body may either be
JSON (in which case a `Content-Type: application/json` header should be
included) or `x-www-form-urlencoded`.

### Read: `GET /:id`
To read a single row/entity from the database, you can do a get request to this
route. The response will be the JSON encoded data for that row in the body.

### Update: `PATCH /:id` or `PUT /:id`
To partially update an entity, only sending the modified fields, send a patch
request to this route. You can also use a put request, in which case all fields
must be provided, including those that cannot be modified. Fields that cannot
be modified must have their original value. The body should either be JSON or
form encoded.

### Full update: `PUT /:id`
To update an entity send a post request to this route. The same constraints as
the create route apply.

### Delete: `DELETE /:id`
Send a delete request to this route to delete an entity from the database.

### List `GET /`
Send a get request to the root route to get a list of all available entities.
Note that the maximum number of items that the list handler will return is
limited. There are several options you can add to the list endpoint for
filtering, sorting and pagination.

- Pagination: to paginate, add the `?offset=n&limit=m` parameters to your
  query. Note that the limit is hard capped, so you can never go over the
  `max_limit` as set in the macro.
- Sorting: to sort, add a `?sort=[field]` query parameter to your query. You
  may add multiple sort query parameters. By default a column will be sorted
  in ascending order. To sort in descending order add a `-` in front of the
  field name. E.g. `?sort=-created` to sort the created field in descending
  order.
- Filtering: to filter, add `?filter[field]op=value` query parameters. The
  field should be in brackets and the operator should be one of a predefined
  list shown below. If you add multiple filters all of those conditions will be
  applied in conjunction (using `AND`). If you want to use the `eq` operation
  you can leave the operator off, in which case you can use
  `?filter[field]=value`. The possible operations are:
  - `eq` (the default operator if you don't provide any operator): the results
    should be filtered such that all values in a specific column equal the
    given value.
  - `ne`: the results should be filtered such that all values in a specific
    column do not match the given value.
  - `lt`, `le`, `gt`, `ge`: less than, less than or equal to, greater than, and
    greater than or equal to operations respectively.
  - `in`: The value should be a comma separated list of values. If one of the
    values in that matches the value in the column then that row shoul be
    included. Note that you cannot use `in` with columns that contain values
    with commas, as there is no escaping mechanism.
- Partials: if you want, you can limit which fields will be returned in a
  response. You can include fields using `include` (in which case all fields
  not mentioned will be excluded automatically) or you can choose to specify
  which fields should be excluded using `exclude`. You can repeat include and
  exclude query parameters to include or exclude multiple fields.
