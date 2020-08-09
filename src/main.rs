use plotters::prelude::*;
use serde_json::{Value, Result, json};
use serde::{Deserialize, Serialize};
use select::document::Document;
use select::predicate::{Class, Name, Predicate, Attr};
use select::node::Node;
use hyper::{Client, Uri};
use scraper::{Html, Selector};
use csv;
use regex::Regex;
use std::{io, process};
use std::fs::File;
use std::fs;
use std::path::Path;
use std::borrow::Borrow;

mod browser;

fn main() {
    let mut browser = browser::Browser::new();
    let result = get_region_vec(&mut browser);
    if (result.is_some()) {
        println!("{:?}", result);
    } else {
        return;
    }
    //TODO 多线程爬取
    for region in result.unwrap() {
        let citys = get_city_vec(&mut browser, &region);
        for city in citys {
            let mut all_data: Vec<FangJia> = Vec::new();
            let file_path = format!("data/{0}.csv", city.py);
            if Path::new(&file_path).exists() {
                println!("{} {} {}", region.name,city.name , "历史记录");
                if true{
                    continue;
                }
                let mut reader = csv::Reader::from_path(&file_path).unwrap();
                let iter = reader.records();
                for row in iter {
                    let r: FangJia = row.map(move |x|
                        {
                            FangJia { date: String::from(x.get(0).unwrap()).replace("年", "-").replace("月", ""), price: x.get(1).unwrap().parse::<i32>().unwrap() }
                        }).unwrap();
                    all_data.push(r);
                }
            } else {
                println!("{} {} {}", region.name,city.name , "网上同步");
                for year in 2009..2021 {
                    all_data.extend(spider(&mut browser, &city, year));
                }
                let mut wtr = csv::Writer::from_path(&file_path).unwrap();
                for record in all_data.clone() {
                    wtr.serialize(record);
                }
                wtr.flush();
            }
            if all_data.len() > 0 {
                draw(&mut all_data, &region, &city);
            }
        }
    }
}

fn crawler_city() {}

//区域
fn get_region_vec(browser: &mut browser::Browser) -> Option<Vec<City>> {
    let url = "https://www.anjuke.com/fangjia/";
    let mut response = browser.get(url, &json!({}));
    if response.status().is_success() {
        let content = response.text().unwrap();
        let document = Html::parse_document(content.as_ref());
        let select = document.select(&Selector::parse(r#"div[class="items"]"#).unwrap()).next()?;
        let mut result: Vec<City> = Vec::new();
        for a in select.select(&Selector::parse("a").unwrap()) {
            match a.value().attr("href") {
                Some(t) => {
                    let city = City {
                        py: t[30..t.len() - 1].to_string(),
                        name: a.text().next().unwrap().to_string(),
                    };
                    result.push(city);
                }
                _ => ()
            };
        }
        return Some(result);
    }
    None
}

//城市
fn get_city_vec(browser: &mut browser::Browser, city: &City) -> Vec<City> {
    let url = format!("https://www.anjuke.com/fangjia/{0}/", city.py);
    let mut response = browser.get(url.as_ref(), &json!({}));
    let mut result: Vec<City> = Vec::new();
    if response.status().is_success() {
        let content = response.text().unwrap();
        let document = Html::parse_document(content.as_ref());
        let sele = document.select(&Selector::parse(r#"div[class="sub-items"]"#).unwrap()).next();
        if sele.is_some() {
            for a in sele.unwrap().select(&Selector::parse("a").unwrap()) {
                let url = a.value().attr("href");
                match url {
                    Some(t) => {
                        let name = t[30..t.len() - 1].to_string();
                        let city = City {
                            py: name,
                            name: a.text().next().unwrap().to_string(),
                        };
                        result.push(city);
                    }
                    _ => ()
                }
            }
        }
    }
    result
}

fn draw(data: &mut Vec<FangJia>, region: &City, city: &City) {
    let max_data = data.iter().max_by_key(|x| x.price).unwrap();
    let min_data = data.iter().min_by_key(|x| x.price).unwrap();
    let file_path = format!("data/{0}-{1}.png", region.py, city.py);
    let root = BitMapBackend::new(&file_path, (2048, 480)).into_drawing_area();
    root.fill(&WHITE);
    let root = root.margin(10, 10, 10, 10);
    let mut chart = ChartBuilder::on(&root)
        .caption(format!("{0} {1} 房价走势图", region.name, city.name).as_str(), ("Hei", 40).into_font())
        .x_label_area_size(20)
        .y_label_area_size(80)
        .build_ranged(0f32..data.len() as f32, match (min_data.price as f32 - ((max_data.price - min_data.price) / 10) as f32) {
            x if x > 0.0 => x,
            _ => 0.0
        }..max_data.price as f32 + ((max_data.price - min_data.price) / 10) as f32).unwrap();
    chart
        .configure_mesh()
        // We can customize the maximum number of labels allowed for each axis
        .x_labels(40)
        .y_labels(5)
        .x_label_formatter(&|x| {
            match x.to_string().parse::<usize>() {
                Ok(i) => {
                    match data.get(i) {
                        Some(d) => d.date.clone(),
                        None => String::new(),
                    }
                }
                _ => String::new()
            }
        })
        .y_label_formatter(&|x| format!("{0}", x))
        .draw().unwrap();
    let mut draw_data: Vec<(f32, f32)> = Vec::new();
    for index in 0..data.len() {
        draw_data.push((index as f32, data[index].price as f32))
    }
    println!("{:?}{:?}", city, draw_data);
    chart.draw_series(LineSeries::new(
        draw_data,
        &RED,
    )).unwrap();
}

fn spider(browser: &mut browser::Browser, city: &City, year: i32) -> Vec<FangJia> {
    let url = format!("https://www.anjuke.com/fangjia/{0}{1}/", city.py, year.to_string());
    let mut res = browser.get(url.as_str(), &json!({}));
    let mut fj: Vec<FangJia> = Vec::new();
    if (res.status() == 200) {
        let mut text = res.text().unwrap();
        let document = Html::parse_document(text.as_str());
        let parse = Selector::parse(r#"div[class="avger clearfix"]"#).unwrap();
        let div = document.select(&parse).next();
        if div.is_some() {
            let parse2 = Selector::parse(r#"div[class="fjlist-box boxstyle2"]"#).unwrap();
            let div = div.unwrap().select(&parse2).next();
            if (div.is_some()) {
                for li in div.unwrap().select(&Selector::parse("li").unwrap()) {
                    let b_ele = li.select(&Selector::parse("b").unwrap()).next();
                    let mut data = FangJia::default();
                    if (b_ele.is_some()) {
                        let mut date = b_ele.unwrap().text();
                        data.date = String::from(date.next().unwrap()).replace("房价", "").replace("年", "-").replace("月", "");
                    }
                    let span_ele = li.select(&Selector::parse("span").unwrap()).next();
                    if (span_ele.is_some()) {
                        let num_regex = Regex::new(r"\d+").unwrap();
                        let price = span_ele.unwrap().text().next().unwrap();
                        let find = num_regex.find(&price);
                        if (find.is_some()) {
                            let x = find.unwrap();
                            data.price = price[x.start()..x.end()].parse::<i32>().unwrap();
                        }
                    }
                    fj.push(data);
                }
            }
        }
    }
    fj.reverse();
    fj
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
struct FangJia {
    date: String,
    price: i32,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
struct City {
    py: String,
    name: String,
}