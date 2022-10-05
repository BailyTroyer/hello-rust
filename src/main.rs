use mysql::prelude::*;
use mysql::*;
use std::io::*;

#[derive(Debug, PartialEq, Eq)]
struct Entity {
    id: i32,
    value: String,
}

fn insert_data() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let url = "mysql://username:password@localhost:3306/db";
    let pool = Pool::new(url)?;

    let mut conn = pool.get_conn()?;

    conn.query_drop("DROP TABLE entity")?;

    // Let's create a table for entities.
    conn.query_drop(
        r"CREATE TABLE IF NOT EXISTS entity (
            id int not null,
            value text
        )",
    )?;

    let entities = vec![Entity {
        id: 1,
        value: "foo".into(),
    }];

    // Now let's insert entities to the database
    conn.exec_batch(
        r"INSERT INTO entity (id, value)
          VALUES (:id, :value)",
        entities.iter().map(|e| {
            params! {
                "id" => e.id,
                "value" => &e.value,
            }
        }),
    )?;

    // Let's select payments from database. Type inference should do the trick here.
    let selected_payments = conn.query_map("SELECT id, value from entity", |(id, value)| {
        Entity { id, value }
    })?;

    println!("entities {:?}", entities);

    // Let's make sure, that `payments` equals to `selected_payments`.
    // Mysql gives no guaranties on order of returned rows
    // without `ORDER BY`, so assume we are lucky.
    assert_eq!(entities, selected_payments);
    println!("Yay!");

    Ok(())
}

fn read_line() -> String {
    let mut s = String::new();
    print!("> ");
    let _ = stdout().flush();
    stdin()
        .read_line(&mut s)
        .expect("Did not enter a correct string");
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }
    s
}

fn main() {
    println!("Hello, world!");
    let _ = insert_data();

    let s = read_line();
    println!("You typed: {}", s);
}
