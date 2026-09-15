use mongodb::bson::{Bson, Document};

/// Универсальный табличный вывод результата агрегации.
/// Каждый пайплайн заканчивается стадией $project, которая явно задаёт
/// порядок и подписи столбцов — поэтому здесь достаточно взять ключи
/// первого документа и вывести значения как строки.
pub fn print_documents(docs: &[Document]) {
    if docs.is_empty() {
        println!("\nНичего не найдено.");
        return;
    }

    let headers: Vec<String> = docs[0].keys().cloned().collect();

    let rows: Vec<Vec<String>> = docs
        .iter()
        .map(|doc| headers.iter().map(|key| render(doc.get(key))).collect())
        .collect();

    let widths: Vec<usize> = headers
        .iter()
        .enumerate()
        .map(|(index, header)| {
            let header_width = header.chars().count();
            let value_width = rows
                .iter()
                .map(|row| row[index].chars().count())
                .max()
                .unwrap_or(0);
            header_width.max(value_width)
        })
        .collect();

    print_separator(&widths);
    print_line(&headers, &widths);
    print_separator(&widths);
    for row in &rows {
        print_line(row, &widths);
    }
    print_separator(&widths);
    println!("Строк: {}", docs.len());
}

fn render(value: Option<&Bson>) -> String {
    match value {
        None | Some(Bson::Null) => "—".to_string(),
        Some(Bson::String(text)) => text.clone(),
        Some(Bson::Int32(number)) => number.to_string(),
        Some(Bson::Int64(number)) => number.to_string(),
        Some(Bson::Double(number)) => format!("{number:.2}"),
        Some(Bson::Boolean(true)) => "да".to_string(),
        Some(Bson::Boolean(false)) => "нет".to_string(),
        Some(Bson::DateTime(date)) => date
            .try_to_rfc3339_string()
            .map(|value| value[..10].to_string())
            .unwrap_or_else(|_| "?".to_string()),
        Some(other) => other.to_string(),
    }
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
