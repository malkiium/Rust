use std::collections::{HashMap, BinaryHeap};
use std::cmp::Ordering;
use std::io::{self, Write};

const CIPHERTEXT: &str = "bxrworn, dodcx iy lbks !";

const COMMON_WORDS: &[&str] = &[
    "the","and","to","of","in","is","it","you","that","he","was","for","on","are",
    "as","with","his","they","be","at","one","have","this","from","or","had","by",
    "but","not","we","my","so","if","me","your","what","all","can","no","about",
    "have","this","will","your","from","they","would","there","their","which","when",
    "make","like","time","very","when","come","just","know","take","people","year",
    "work","back","call","come","feel","find","give","good","hand","high","keep",
    "last","life","live","make","mean","need","next","open","over","part","play",
    "said","same","seem","such","tell","than","that","them","then","these","they",
    "this","thus","time","very","want","well","were","what","when","will","with",
    "word","work","would","write","years","ancient","library","stood","majestically",
    "hillside","weathered","stone","walls","holding","countless","secrets","within",
    "scholars","across","kingdom","journey","months","access","rare","manuscripts",
    "precious","knowledge","head","librarian","guarded","treasures","fiercely",
    "allowing","dedicated","researchers","study","carefully"
];

const FREQ: &str = "etaoinshrdlu";

#[derive(Clone, Eq, PartialEq)]
struct Result {
    score: i32,
    cipher_type: String,
    params: String,
    plaintext_preview: String,
    plaintext_full: String,
}

impl Ord for Result {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}

impl PartialOrd for Result {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct TopN {
    heap: BinaryHeap<Result>,
    limit: usize,
}

impl TopN {
    fn new(limit: usize) -> Self {
        TopN {
            heap: BinaryHeap::new(),
            limit,
        }
    }

    fn insert(&mut self, result: Result) {
        if self.heap.len() < self.limit {
            self.heap.push(result);
        } else if result.score > self.heap.peek().unwrap().score {
            self.heap.pop();
            self.heap.push(result);
        }
    }

    fn insert_lightweight(&mut self, score: i32, cipher_type: String, params: String, plaintext: &str) {
        let preview = plaintext[..plaintext.len().min(80)].to_string();
        self.insert(Result {
            score,
            cipher_type,
            params,
            plaintext_preview: preview,
            plaintext_full: plaintext.to_string(),
        });
    }

    fn into_sorted_vec(self) -> Vec<Result> {
        let mut vec: Vec<Result> = self.heap.into();
        vec.sort_by(|a, b| b.score.cmp(&a.score));
        vec
    }

    fn best_result(&self) -> Option<&Result> {
        self.heap.peek()
    }
}

// ========== DICTIONARY VALIDATION ==========

fn is_valid_english(text: &str) -> i32 {
    let words: Vec<String> = text
        .split(|c: char| !c.is_ascii_alphabetic())
        .map(|w| w.to_lowercase())
        .filter(|w| w.len() >= 3)
        .collect();

    if words.is_empty() {
        return 0;
    }

    let valid_count = words.iter().filter(|w| COMMON_WORDS.contains(&w.as_str())).count();
    (valid_count as i32 * 100) / words.len() as i32
}

// ========== DECRYPTION FUNCTIONS ==========

fn decrypt_caesar(text: &str, shift: u8) -> String {
    text.chars()
        .map(|c| {
            if c.is_ascii_lowercase() {
                ((c as u8 - b'a' + 26 - shift) % 26 + b'a') as char
            } else if c.is_ascii_uppercase() {
                ((c as u8 - b'A' + 26 - shift) % 26 + b'A') as char
            } else {
                c
            }
        })
        .collect()
}

fn decrypt_rot13(text: &str) -> String {
    decrypt_caesar(text, 13)
}

fn decrypt_atbash(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_ascii_lowercase() {
                (b'z' - (c as u8 - b'a')) as char
            } else if c.is_ascii_uppercase() {
                (b'Z' - (c as u8 - b'A')) as char
            } else {
                c
            }
        })
        .collect()
}

fn decrypt_vigenere(text: &str, key: &[u8]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut k = 0;

    for c in text.chars() {
        if c.is_ascii_alphabetic() {
            let shift = key[k % key.len()];
            let base = if c.is_ascii_lowercase() { b'a' } else { b'A' };
            let d = ((c as u8 - base + 26 - shift) % 26) + base;
            out.push(d as char);
            k += 1;
        } else {
            out.push(c);
        }
    }

    out
}

fn decrypt_rail_fence(text: &str, rails: usize) -> String {
    if rails <= 1 {
        return text.to_string();
    }

    let n = text.len();
    let cipher_chars: Vec<char> = text.chars().collect();
    let mut fence: Vec<Vec<usize>> = vec![vec![]; rails];
    let mut rail = 0;
    let mut direction = 1;

    for i in 0..n {
        fence[rail].push(i);
        if rail == 0 {
            direction = 1;
        } else if rail == rails - 1 {
            direction = -1;
        }
        rail = (rail as i32 + direction) as usize;
    }

    let mut result = vec!['?'; n];
    let mut cipher_idx = 0;

    for rail_chars in fence.iter() {
        for &pos in rail_chars {
            if cipher_idx < cipher_chars.len() {
                result[pos] = cipher_chars[cipher_idx];
                cipher_idx += 1;
            }
        }
    }

    result.iter().collect()
}

fn decrypt_affine(text: &str, a: u8, b: u8) -> String {
    text.chars()
        .map(|c| {
            if c.is_ascii_lowercase() {
                let x = (c as u8 - b'a') as u32;
                let y = ((mod_inverse(a as u32, 26) * (x + b as u32)) % 26) as u8;
                (b'a' + y) as char
            } else if c.is_ascii_uppercase() {
                let x = (c as u8 - b'A') as u32;
                let y = ((mod_inverse(a as u32, 26) * (x + b as u32)) % 26) as u8;
                (b'A' + y) as char
            } else {
                c
            }
        })
        .collect()
}

fn mod_inverse(mut a: u32, mut m: u32) -> u32 {
    let m0 = m;
    let (mut x0, mut x1) = (0i32, 1i32);

    if m == 1 {
        return 0;
    }

    while a > 1 {
        let q = (a / m) as i32;
        let t = m as i32;

        m = (a % m) as u32;
        a = t as u32;
        let t = x0;
        x0 = x1 - q * x0;
        x1 = t;
    }

    if x1 < 0 {
        (x1 + m0 as i32) as u32
    } else {
        x1 as u32
    }
}

fn decrypt_beaufort(text: &str, key: &[u8]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut k = 0;

    for c in text.chars() {
        if c.is_ascii_alphabetic() {
            let shift = key[k % key.len()];
            let base = if c.is_ascii_lowercase() { b'a' } else { b'A' };
            let d = ((shift + 26 - (c as u8 - base)) % 26) + base;
            out.push(d as char);
            k += 1;
        } else {
            out.push(c);
        }
    }

    out
}

fn decrypt_columnar_transposition(text: &str, key: &str) -> String {
    let cols = key.len();
    let rows = (text.len() + cols - 1) / cols;
    let chars: Vec<char> = text.chars().collect();

    let mut key_indices: Vec<usize> = (0..cols).collect();
    key_indices.sort_by_key(|&i| key.chars().nth(i).unwrap());

    let mut result = vec!['?'; text.len()];
    let mut read_idx = 0;

    for original_pos in key_indices.iter() {
        for row in 0..rows {
            let pos = row * cols + original_pos;
            if pos < text.len() && read_idx < chars.len() {
                result[pos] = chars[read_idx];
                read_idx += 1;
            }
        }
    }

    result.iter().collect()
}

fn decrypt_playfair(text: &str, key: &str) -> String {
    let key_lower = key.to_lowercase().replace('j', "i");
    let mut keytable = String::new();
    let mut seen = std::collections::HashSet::new();
    
    for c in key_lower.chars() {
        if c.is_ascii_alphabetic() && !seen.contains(&c) {
            keytable.push(c);
            seen.insert(c);
        }
    }
    
    for c in 'a'..='z' {
        if c != 'j' && !seen.contains(&c) {
            keytable.push(c);
        }
    }

    let mut result = String::new();
    let clean_text: String = text.chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| if c.to_ascii_lowercase() == 'j' { 'i' } else { c.to_ascii_lowercase() })
        .collect();

    for i in (0..clean_text.len()).step_by(2) {
        if i + 1 < clean_text.len() {
            let c1 = clean_text.chars().nth(i).unwrap();
            let c2 = clean_text.chars().nth(i + 1).unwrap();
            
            let pos1 = keytable.find(c1).unwrap_or(0);
            let pos2 = keytable.find(c2).unwrap_or(0);
            
            let row1 = pos1 / 5;
            let col1 = pos1 % 5;
            let row2 = pos2 / 5;
            let col2 = pos2 % 5;
            
            if row1 == row2 {
                let new_col1 = (col1 + 4) % 5;
                let new_col2 = (col2 + 4) % 5;
                result.push(keytable.chars().nth(row1 * 5 + new_col1).unwrap());
                result.push(keytable.chars().nth(row2 * 5 + new_col2).unwrap());
            } else if col1 == col2 {
                let new_row1 = (row1 + 4) % 5;
                let new_row2 = (row2 + 4) % 5;
                result.push(keytable.chars().nth(new_row1 * 5 + col1).unwrap());
                result.push(keytable.chars().nth(new_row2 * 5 + col2).unwrap());
            } else {
                result.push(keytable.chars().nth(row1 * 5 + col2).unwrap());
                result.push(keytable.chars().nth(row2 * 5 + col1).unwrap());
            }
        }
    }

    result
}

fn decrypt_polybius_square(text: &str) -> String {
    let polybius = "abcdefghiklmnopqrstuvwxyz";
    let mut result = String::new();
    let clean_text: String = text.chars().filter(|c| c != &' ').collect();

    for i in (0..clean_text.len()).step_by(2) {
        if i + 1 < clean_text.len() {
            let c1 = clean_text.chars().nth(i).unwrap();
            let c2 = clean_text.chars().nth(i + 1).unwrap();
            
            if "12345".contains(c1) && "12345".contains(c2) {
                let row = "12345".find(c1).unwrap_or(0);
                let col = "12345".find(c2).unwrap_or(0);
                let idx = row * 5 + col;
                if idx < polybius.len() {
                    result.push(polybius.chars().nth(idx).unwrap());
                }
            }
        }
    }

    result
}

fn decrypt_bacon(text: &str) -> String {
    let bacon_map = vec![
        ('a', "aaaaa"), ('b', "aaaab"), ('c', "aaaba"), ('d', "aaabb"), ('e', "aabaa"),
        ('f', "aabab"), ('g', "aabba"), ('h', "aabbb"), ('i', "abaaa"), ('j', "abaab"),
        ('k', "ababa"), ('l', "ababb"), ('m', "abbaa"), ('n', "abbab"), ('o', "abbba"),
        ('p', "abbbb"), ('q', "baaaa"), ('r', "baaab"), ('s', "baaba"), ('t', "baabb"),
        ('u', "babaa"), ('v', "babab"), ('w', "babba"), ('x', "babbb"), ('y', "bbaaa"),
        ('z', "bbaab"),
    ];

    let clean_text: String = text.chars().filter(|c| c.is_ascii_alphabetic()).collect();
    let mut result = String::new();

    for i in (0..clean_text.len()).step_by(5) {
        let chunk: String = clean_text.chars().skip(i).take(5).collect();
        if chunk.len() == 5 {
            for (letter, code) in &bacon_map {
                if &chunk == code {
                    result.push(*letter);
                    break;
                }
            }
        }
    }

    result
}

fn decrypt_reverse(text: &str) -> String {
    text.chars().rev().collect()
}

fn decrypt_atbash_vigenere(text: &str, key: &[u8]) -> String {
    let atbash_text = decrypt_atbash(text);
    decrypt_vigenere(&atbash_text, key)
}

// ========== SCORING FUNCTION ==========

fn score_english(text: &str) -> i32 {
    let mut freq = HashMap::new();

    for c in text.chars().filter(|c| c.is_ascii_alphabetic()) {
        *freq.entry(c.to_ascii_lowercase()).or_insert(0) += 1;
    }

    let mut score = 0;

    for c in FREQ.chars() {
        score += freq.get(&c).unwrap_or(&0);
    }

    for w in text
        .split(|c: char| !c.is_ascii_alphabetic())
        .map(|w| w.to_ascii_lowercase())
        .filter(|w| w.len() >= 3)
    {
        if COMMON_WORDS.contains(&w.as_str()) {
            score += 10;
        }
    }

    // Dictionary validation bonus
    let dict_score = is_valid_english(text);
    score += dict_score * 2;

    score
}

fn get_user_choice() -> usize {
    loop {
        println!("\n════════════════════════════════════════════════");
        println!("Choose a cipher to test (or 0 to test all):");
        println!(" 1. Caesar Cipher");
        println!(" 2. ROT13");
        println!(" 3. Atbash");
        println!(" 4. Vigenère Cipher");
        println!(" 5. Rail Fence Cipher");
        println!(" 6. Affine Cipher");
        println!(" 7. Beaufort Cipher");
        println!(" 8. Columnar Transposition");
        println!(" 9. Playfair Cipher");
        println!("10. Polybius Square");
        println!("11. Bacon Cipher");
        println!("12. Reverse Cipher");
        println!("13. Atbash + Vigenère Hybrid");
        println!(" 0. Test ALL ciphers (Brute Force All)");
        print!("\nYour choice (0-13): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        match input.trim().parse::<usize>() {
            Ok(choice) if choice <= 13 => return choice,
            _ => println!("Invalid choice. Please enter a number between 0 and 13."),
        }
    }
}

fn crack_specific_cipher(choice: usize, top_n: &mut TopN) -> bool {
    println!("\n🔍 Attempting to crack with chosen cipher...");
    
    match choice {
        1 => {
            println!("Testing Caesar cipher (all 26 shifts)...");
            for shift in 0..26 {
                let plain = decrypt_caesar(CIPHERTEXT, shift);
                let score = score_english(&plain);
                top_n.insert_lightweight(score, "Caesar".to_string(), format!("shift {}", shift), &plain);
            }
        }
        2 => {
            println!("Testing ROT13...");
            let plain = decrypt_rot13(CIPHERTEXT);
            let score = score_english(&plain);
            top_n.insert_lightweight(score, "ROT13".to_string(), "ROT13".to_string(), &plain);
        }
        3 => {
            println!("Testing Atbash cipher...");
            let plain = decrypt_atbash(CIPHERTEXT);
            let score = score_english(&plain);
            top_n.insert_lightweight(score, "Atbash".to_string(), "Atbash".to_string(), &plain);
        }
        4 => {
            println!("Testing Vigenère cipher (1-5 char keys)...");
            for len in 1..=5 {
                let total = 26_usize.pow(len as u32);
                let mut key = vec![0u8; len];
                println!("  Trying {}-character keys...", len);

                for i in 0..total {
                    let mut n = i;
                    for j in (0..len).rev() {
                        key[j] = (n % 26) as u8;
                        n /= 26;
                    }

                    let plain = decrypt_vigenere(CIPHERTEXT, &key);
                    let score = score_english(&plain);
                    let k: String = key.iter().map(|&x| (b'a' + x) as char).collect();
                    
                    top_n.insert_lightweight(score, "Vigenère".to_string(), format!("key: {}", k), &plain);
                }
            }
        }
        5 => {
            println!("Testing Rail Fence cipher (2-15 rails)...");
            for rails in 2..=15 {
                let plain = decrypt_rail_fence(CIPHERTEXT, rails);
                let score = score_english(&plain);
                top_n.insert_lightweight(score, "Rail Fence".to_string(), format!("{} rails", rails), &plain);
            }
        }
        6 => {
            println!("Testing Affine cipher (all combinations)...");
            let coprime_a = vec![1, 3, 5, 7, 9, 11, 15, 17, 19, 21, 23, 25];
            for &a in &coprime_a {
                for b in 0..26 {
                    let plain = decrypt_affine(CIPHERTEXT, a, b);
                    let score = score_english(&plain);
                    top_n.insert_lightweight(score, "Affine".to_string(), format!("a={}, b={}", a, b), &plain);
                }
            }
        }
        7 => {
            println!("Testing Beaufort cipher (1-5 char keys)...");
            for len in 1..=5 {
                let total = 26_usize.pow(len as u32);
                let mut key = vec![0u8; len];
                println!("  Trying {}-character keys...", len);

                for i in 0..total {
                    let mut n = i;
                    for j in (0..len).rev() {
                        key[j] = (n % 26) as u8;
                        n /= 26;
                    }

                    let plain = decrypt_beaufort(CIPHERTEXT, &key);
                    let score = score_english(&plain);
                    let k: String = key.iter().map(|&x| (b'a' + x) as char).collect();
                    
                    top_n.insert_lightweight(score, "Beaufort".to_string(), format!("key: {}", k), &plain);
                }
            }
        }
        8 => {
            println!("Testing Columnar Transposition (2-10 cols)...");
            for cols in 2..=10 {
                let mut key = String::new();
                for i in 0..cols {
                    key.push((b'a' + (i as u8)) as char);
                }
                let plain = decrypt_columnar_transposition(CIPHERTEXT, &key);
                let score = score_english(&plain);
                top_n.insert_lightweight(score, "Columnar".to_string(), format!("{} cols", cols), &plain);
            }
        }
        9 => {
            println!("Testing Playfair cipher (common keys)...");
            let keys = vec!["key", "secret", "cipher", "enigma", "cryptography", "library", "ancient", "knowledge"];
            for key in keys {
                let plain = decrypt_playfair(CIPHERTEXT, key);
                let score = score_english(&plain);
                top_n.insert_lightweight(score, "Playfair".to_string(), format!("key: {}", key), &plain);
            }
        }
        10 => {
            println!("Testing Polybius Square...");
            let plain = decrypt_polybius_square(CIPHERTEXT);
            let score = score_english(&plain);
            top_n.insert_lightweight(score, "Polybius".to_string(), "Polybius Square".to_string(), &plain);
        }
        11 => {
            println!("Testing Bacon cipher...");
            let plain = decrypt_bacon(CIPHERTEXT);
            let score = score_english(&plain);
            top_n.insert_lightweight(score, "Bacon".to_string(), "Bacon".to_string(), &plain);
        }
        12 => {
            println!("Testing Reverse cipher...");
            let plain = decrypt_reverse(CIPHERTEXT);
            let score = score_english(&plain);
            top_n.insert_lightweight(score, "Reverse".to_string(), "Reverse".to_string(), &plain);
        }
        13 => {
            println!("Testing Atbash + Vigenère Hybrid (1-4 char keys)...");
            for len in 1..=4 {
                let total = 26_usize.pow(len as u32);
                let mut key = vec![0u8; len];
                println!("  Trying {}-character keys...", len);

                for i in 0..total {
                    let mut n = i;
                    for j in (0..len).rev() {
                        key[j] = (n % 26) as u8;
                        n /= 26;
                    }

                    let plain = decrypt_atbash_vigenere(CIPHERTEXT, &key);
                    let score = score_english(&plain);
                    let k: String = key.iter().map(|&x| (b'a' + x) as char).collect();
                    
                    top_n.insert_lightweight(score, "Hybrid".to_string(), format!("key: {}", k), &plain);
                }
            }
        }
        _ => return false,
    }
    
    true
}

fn crack_all_ciphers(top_n: &mut TopN) {
    println!("\n🔍 Brute forcing ALL ciphers...");
    
    // Test all ciphers
    for i in 1..=13 {
        crack_specific_cipher(i, top_n);
    }
}

fn display_results(top_n: &TopN, found_exact: bool) {
    let results = top_n.heap.iter().cloned().collect::<Vec<_>>();
    
    if results.is_empty() {
        println!("\n❌ No results found!");
        return;
    }
    
    if found_exact {
        println!("\n✅ SUCCESS! Found the correct decryption!");
    } else {
        println!("\n❌ No exact match found. Here are the best candidates:");
    }
    
    println!("\n🏆 TOP RESULTS:");
    println!("{}════════════════════════════════════════════════", "═".repeat(25));
    
    for (rank, result) in results.iter().enumerate() {
        println!("  #{:<2} | Score: {:<4} | Type: {:<15} | Params: {}", 
                 rank + 1, result.score, result.cipher_type, result.params);
        println!("       └─ {}\n", &result.plaintext_preview);
    }
    
    // Show the best candidate in full
    if let Some(best) = top_n.best_result() {
        println!("\n📝 BEST CANDIDATE:");
        println!("{}════════════════════════════════════════════════", "═".repeat(25));
        println!("Cipher: {} | Params: {}", best.cipher_type, best.params);
        println!("Score: {}", best.score);
        println!("\nDecrypted text:");
        println!("{}\n", best.plaintext_full);
    }
}

fn main() {
    println!("\n╔════════════════════════════════════════╗");
    println!("║         CRYPTO BREAKER GAME           ║");
    println!("╚════════════════════════════════════════╝\n");
    println!("Ciphertext to crack:\n");
    println!("  \"{}\"\n", CIPHERTEXT);
    
    loop {
        let choice = get_user_choice();
        let mut top_n = TopN::new(5);
        let mut found_exact = false;
        
        if choice == 0 {
            println!("\n🚀 Starting full brute force attack on all ciphers...");
            crack_all_ciphers(&mut top_n);
        } else {
            println!("\n🎯 Testing cipher #{}...", choice);
            found_exact = crack_specific_cipher(choice, &mut top_n);
        }
        
        display_results(&top_n, found_exact);
        
        // Ask if user wants to try another cipher
        println!("{}════════════════════════════════════════════════", "═".repeat(25));
        print!("\nTry another cipher? (y/n): ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        if input.trim().to_lowercase() != "y" {
            println!("\n👋 Thanks for playing!");
            break;
        }
    }
}