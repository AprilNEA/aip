use aip_160::parse_filter;

fn main() {

    // Example filters
    let examples = vec![
        "name = \"John Doe\"",
        "age > 18 AND active = true",
        "(status = \"active\" OR status = \"pending\") AND created > \"2024-01-01\"",
        "NOT deleted = true",
        "tags : \"important\"",
        "price >= 100 AND price <= 1000",
        "email : \"@example.com\" AND verified = true",
        "count != 0",
        "value = null",
    ];

    for (i, example) in examples.iter().enumerate() {
        println!("Example {}: {}", i + 1, example);

        match parse_filter(example) {
            Ok(filter) => {
                println!("  ✓ Parsed successfully");
                println!("  AST: {:?}", filter.expression);
                println!();
            }
            Err(e) => {
                println!("  ✗ Parse error: {}", e);
                println!();
            }
        }
    }
}
