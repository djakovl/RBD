use postgres::Row;

pub fn print_rows(rows: &[Row]) {
    if rows.is_empty() {
        println!("\nНичего не найдено.");
        return;
    }

    let headers: Vec<String> = rows[0]
        .columns()
        .iter()
        .map(|column| column.name().to_string())
        .collect();

    let data: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            (0..headers.len())
                .map(|index| row.get::<usize, String>(index))
                .collect()
        })
        .collect();

    let widths: Vec<usize> = (0..headers.len())
        .map(|column| {
            let header_width = headers[column].chars().count();
            let value_width = data
                .iter()
                .map(|row| row[column].chars().count())
                .max()
                .unwrap_or(0);
            header_width.max(value_width)
        })
        .collect();

    print_separator(&widths);
    print_line(&headers, &widths);
    print_separator(&widths);
    for row in &data {
        print_line(row, &widths);
    }
    print_separator(&widths);
    println!("Строк: {}", rows.len());
}

fn print_separator(widths: &[usize]) {
    print!("+");
    for width in widths {
        print!("{}+", "-".repeat(width + 2));
    }
    println!();
}

fn print_line(values: &[String], widths: &[usize]) {
    print!("|");
    for (value, width) in values.iter().zip(widths) {
        let padding = width.saturating_sub(value.chars().count());
        print!(" {}{} |", value, " ".repeat(padding));
    }
    println!();
}
