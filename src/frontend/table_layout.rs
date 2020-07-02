// Ported from https://github.com/capptions/column-widths.js/blob/master/index.js

const NARROWS: &str = "!ifjl,;.:-|\n\r\t\0\x0B";
const WIDES: &str = "wm—G@";

#[allow(unused)]
struct Column {
    max: f32,
    sum: f32,
    count: u32,
    widths: Vec<f32>,
    avg: f32,
    sd: f32,
    cv: f32,
    sdmax: f32,
    calc: f32,
}

fn text_width(line: &str) -> f32 {
    line.chars().map(|c| {
        if NARROWS.contains(c) {
            0.4
        } else if WIDES.contains(c) {
            1.3
        } else {
            1.0
        }
    }).sum::<f32>().max(1.0)
}

pub fn column_widths(column_lines: Vec<Vec<String>>) -> Vec<f32> {
    let min_percentage = 100.0 / 3.0 / column_lines.len() as f32;

    let mut columns = Vec::new();

    for column in column_lines {
        let mut max = 0.0f32;
        let mut sum = 0.0;
        let mut count = 0;
        let mut widths = Vec::new();

        for line in column {
            let width = text_width(&line);
            max = max.max(width);
            sum += width;
            count += 1;
            widths.push(width);
        }

        // calculate
        let avg = sum / count as f32;
        let avg_square = widths.iter()
            .map(|width| width - avg)
            .map(|diff| diff * diff)
            .sum::<f32>() / count as f32;
        let sd = avg_square.sqrt();
        let cv = sd / avg;
        let sdmax = sd / max;
        let calc = if (sdmax < 0.3 || cv == 1.0) && (cv == 0.0 || (cv > 0.6 && cv < 1.5)) {
            avg
        } else {
            let mut calc = avg + (max / avg) * 2.0 / (1.0 - cv).abs();
            if calc > max {
                let tmp = if cv > 1.0 && sd > 4.5 && sdmax > 0.2 { (max - avg) / 2.0 } else { 0.0 };
                calc = max - tmp;
            }
            calc
        };

        columns.push(Column { max, sum, count, widths, avg, sd, cv, sdmax, calc });
    }

    let total = columns.iter().map(|col| col.calc).sum::<f32>();
    let mut percentages: Vec<f32> = columns.iter().map(|col| 100.0 / (total / col.calc)).collect();

    for (i, _) in columns.iter().enumerate() {
        let short = min_percentage - percentages[i];

        if short < 0.0 {
            continue;
        }

        let mut lowest_distance = std::f32::MAX;
        let steal_column_idx = columns.iter()
            .enumerate()
            .filter(|(_, col)| {
                let distance = (1.0 - col.cv).abs();
                let should_steal = distance < lowest_distance
                    && col.calc - short > col.avg
                    && percentages[i] - min_percentage >= short;
                if should_steal {
                    lowest_distance = distance;
                }
                should_steal
            }).map(|(i, _)| i)
            .last()
            .unwrap_or_else(|| {
                percentages.iter().enumerate().max_by(|&(_, p1), &(_, p2)| p1.partial_cmp(p2).unwrap()).unwrap().0
            });
        percentages[steal_column_idx] -= short;
        percentages[i] = min_percentage;
    }

    percentages
}