pub fn sparkline(data: &[u64]) -> String {
    let levels = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    if data.is_empty() {
        return " ".repeat(12);
    }

    data.iter()
        .map(|value| {
            let index = (*value).min(8) as usize;
            levels[index]
        })
        .collect()
}
