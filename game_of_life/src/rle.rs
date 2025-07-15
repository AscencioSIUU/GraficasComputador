pub fn parse_rle(rle: &str, offset_x: usize, offset_y: usize) -> Vec<(usize, usize)> {
     let mut result = Vec::new();
     let mut x = 0;
     let mut y = 0;
     let mut count = 0;
 
     for line in rle.lines() {
         if line.starts_with('#') || line.starts_with("x") || line.trim().is_empty() {
             continue;
         }
 
         let chars: Vec<char> = line.chars().collect();
         let mut i = 0;
 
         while i < chars.len() {
             let ch = chars[i];
 
             if ch.is_digit(10) {
                 let mut num_str = ch.to_string();
                 while i + 1 < chars.len() && chars[i + 1].is_digit(10) {
                     i += 1;
                     num_str.push(chars[i]);
                 }
                 count = num_str.parse::<usize>().unwrap();
             } else {
                 let run = if count == 0 { 1 } else { count };
 
                 match ch {
                     'b' => x += run,
                     'o' => {
                         for dx in 0..run {
                             result.push((offset_x + x + dx, offset_y + y));
                         }
                         x += run;
                     }
                     '$' => {
                         y += if count == 0 { 1 } else { count };
                         x = 0;
                     }
                     '!' => break,
                     _ => {}
                 }
 
                 count = 0;
             }
 
             i += 1;
         }
     }
 
     result
 }
 