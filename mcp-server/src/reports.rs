use crate::database::get_complete_lottery_data;
use crate::types::PrizeNumberRow;
use rusqlite::Connection;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn get_category_display_name(category: &str) -> &str {
    match category {
        "first" => "รางวัลที่ 1",
        "second" => "รางวัลที่ 2",
        "third" => "รางวัลที่ 3",
        "fourth" => "รางวัลที่ 4",
        "fifth" => "รางวัลที่ 5",
        "last2" => "รางวัลท้าย 2 ตัว",
        "last3f" => "รางวัลท้าย 3 ตัว (หน้า)",
        "last3b" => "รางวัลท้าย 3 ตัว (หลัง)",
        "near1" => "รางวัลใกล้เคียงรางวัลที่ 1",
        _ => category,
    }
}

pub fn format_prize_amount(amount: &str) -> String {
    if let Ok(num) = amount.parse::<f64>() {
        format!("{:.0} บาท", num)
    } else {
        format!("{} บาท", amount)
    }
}

pub fn generate_html_report(conn: &Connection, date: &str) -> Result<String, Box<dyn Error>> {
    let lottery_data = get_complete_lottery_data(conn, date)?;

    if let Some((lottery, prizes)) = lottery_data {
        let mut category_groups: HashMap<String, Vec<&PrizeNumberRow>> = HashMap::new();

        for prize in &prizes {
            category_groups
                .entry(prize.category.clone())
                .or_insert_with(Vec::new)
                .push(prize);
        }

        let mut html = String::new();

        html.push_str(&format!(
            r#"
<!DOCTYPE html>
<html lang="th">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>ผลการออกรางวัลสลากกินแบ่งรัฐบาล - {}</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, sans-serif;
            margin: 0;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            border-radius: 15px;
            box-shadow: 0 20px 40px rgba(0,0,0,0.1);
            overflow: hidden;
        }}
        .header {{
            background: linear-gradient(135deg, #ff6b6b, #feca57);
            color: white;
            padding: 30px;
            text-align: center;
        }}
        .header h1 {{
            margin: 0;
            font-size: 2.5em;
            font-weight: 700;
            text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
        }}
        .header .date {{
            font-size: 1.3em;
            margin-top: 10px;
            opacity: 0.9;
        }}
        .content {{
            padding: 30px;
        }}
        .prize-section {{
            margin-bottom: 40px;
            border-radius: 10px;
            overflow: hidden;
            box-shadow: 0 4px 15px rgba(0,0,0,0.1);
        }}
        .prize-header {{
            background: linear-gradient(135deg, #4834d4, #686de0);
            color: white;
            padding: 20px;
            font-size: 1.4em;
            font-weight: 600;
        }}
        .prize-header .amount {{
            float: right;
            font-size: 0.9em;
            opacity: 0.9;
        }}
        .prize-numbers {{
            background: #f8f9ff;
            padding: 20px;
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
            gap: 15px;
        }}
        .number-card {{
            background: white;
            padding: 15px;
            text-align: center;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
            border: 2px solid #e2e8f0;
            transition: all 0.3s ease;
        }}
        .number-card:hover {{
            transform: translateY(-2px);
            box-shadow: 0 4px 15px rgba(0,0,0,0.15);
            border-color: #4834d4;
        }}
        .number-value {{
            font-size: 1.8em;
            font-weight: 700;
            color: #2d3748;
            font-family: 'Courier New', monospace;
        }}
        .round-info {{
            font-size: 0.8em;
            color: #718096;
            margin-top: 5px;
        }}
        .first-prize .number-card {{
            background: linear-gradient(135deg, #ffd700, #ffed4e);
            border-color: #d69e2e;
        }}
        .first-prize .number-value {{
            color: #744210;
            font-size: 2.5em;
        }}
        .special-prize .prize-header {{
            background: linear-gradient(135deg, #e53e3e, #fc8181);
        }}
        .special-prize .number-card {{
            background: linear-gradient(135deg, #fed7d7, #fbb6ce);
        }}
        .footer {{
            background: #2d3748;
            color: white;
            padding: 20px;
            text-align: center;
            font-size: 0.9em;
        }}
        .stats {{
            background: #edf2f7;
            padding: 20px;
            margin-bottom: 30px;
            border-radius: 10px;
        }}
        .stats h3 {{
            margin-top: 0;
            color: #2d3748;
        }}
        .stats-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 15px;
            margin-top: 15px;
        }}
        .stat-item {{
            background: white;
            padding: 15px;
            border-radius: 8px;
            text-align: center;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        .stat-number {{
            font-size: 2em;
            font-weight: 700;
            color: #4834d4;
        }}
        .stat-label {{
            color: #718096;
            font-size: 0.9em;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🎰 ผลการออกรางวัลสลากกินแบ่งรัฐบาล</h1>
            <div class="date">งวดวันที่ {}</div>
            <div class="date">Period: {}</div>
        </div>
        
        <div class="content">
            <div class="stats">
                <h3>📊 สถิติรางวัล</h3>
                <div class="stats-grid">
                    <div class="stat-item">
                        <div class="stat-number">{}</div>
                        <div class="stat-label">จำนวนรางวัลทั้งหมด</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-number">{}</div>
                        <div class="stat-label">ประเภทรางวัล</div>
                    </div>
                </div>
            </div>
"#,
            lottery.draw_date,
            lottery.draw_date,
            lottery.period,
            prizes.len(),
            category_groups.len()
        ));

        let category_order = [
            "first", "near1", "second", "third", "fourth", "fifth", "last3f", "last3b", "last2",
        ];

        for category in category_order {
            if let Some(numbers) = category_groups.get(category) {
                if !numbers.is_empty() {
                    let display_name = get_category_display_name(category);
                    let prize_amount = format_prize_amount(&numbers[0].prize_amount);

                    let section_class = if category == "first" {
                        "prize-section first-prize"
                    } else if category == "near1"
                        || category == "last2"
                        || category == "last3f"
                        || category == "last3b"
                    {
                        "prize-section special-prize"
                    } else {
                        "prize-section"
                    };

                    html.push_str(&format!(
                        r#"
            <div class="{}">
                <div class="prize-header">
                    {} 
                    <span class="amount">{}</span>
                </div>
                <div class="prize-numbers">
"#,
                        section_class, display_name, prize_amount
                    ));

                    let mut sorted_numbers = numbers.clone();
                    sorted_numbers.sort_by_key(|n| n.round_number);

                    for number in sorted_numbers {
                        html.push_str(&format!(
                            r#"
                    <div class="number-card">
                        <div class="number-value">{}</div>
                        <div class="round-info">ชุดที่ {}</div>
                    </div>
"#,
                            number.number_value, number.round_number
                        ));
                    }

                    html.push_str("                </div>\n            </div>\n");
                }
            }
        }

        html.push_str(&format!(
            r#"
        </div>
        
        <div class="footer">
            <p>📅 รายงานสร้างเมื่อ: {}</p>
            <p>🔗 ข้อมูลจาก: สำนักงานสลากกินแบ่งรัฐบาล</p>
        </div>
    </div>
</body>
</html>
"#,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ));

        Ok(html)
    } else {
        Err(format!("ไม่พบข้อมูลรางวัลสำหรับวันที่ {}", date).into())
    }
}

pub fn save_html_report(html_content: &str, filename: &str) -> Result<(), Box<dyn Error>> {
    let filepath = Path::new("reports").join(filename);
    let mut file = File::create(&filepath)?;
    file.write_all(html_content.as_bytes())?;
    Ok(())
}

pub fn save_html_report_to_path(
    html_content: &str,
    filename: &str,
    report_path: &str,
) -> Result<(), Box<dyn Error>> {
    let filepath = Path::new(report_path).join(filename);

    // Ensure the report directory exists
    if let Some(parent) = filepath.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = File::create(&filepath)?;
    file.write_all(html_content.as_bytes())?;
    Ok(())
}

pub fn generate_and_save_report(conn: &Connection, date: &str) -> Result<(), Box<dyn Error>> {
    match generate_html_report(conn, date) {
        Ok(html_content) => {
            let filename = format!("lottery_report_{}.html", date);
            save_html_report(&html_content, &filename)?;

            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub fn generate_and_save_report_to_path(
    conn: &Connection,
    date: &str,
    report_path: &str,
) -> Result<(), Box<dyn Error>> {
    match generate_html_report(conn, date) {
        Ok(html_content) => {
            let filename = format!("lottery_report_{}.html", date);
            save_html_report_to_path(&html_content, &filename, report_path)?;

            Ok(())
        }
        Err(e) => Err(e),
    }
}
