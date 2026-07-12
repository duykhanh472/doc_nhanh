use chrono::Utc;
use clap::Parser;
use scraper::{Html, Selector};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use url::Url;
use zip::CompressionMethod;
use zip::write::SimpleFileOptions;

/// Công cụ CLI tải bài viết từ Internet và đóng gói thành tệp EPUB để đọc offline.
#[derive(Parser, Debug)]
#[command(name = "doc_nhanh")]
#[command(author = "Your Name <your.email@example.com>")]
#[command(version = "0.1.0")]
#[command(about = "Tải bài viết và đóng gói thành tệp EPUB chuẩn chỉnh", long_about = None)]
struct Args {
    /// Chế độ hoạt động: 'web' (cào link từ file) hoặc 'md' (chuyển đổi thư mục markdown)
    #[arg(short, long, default_value = "web")]
    mode: String,

    /// Đường dẫn tới tệp plaintext chứa danh sách URL (mỗi dòng một URL)
    #[arg(short = 'i', long = "input", default_value = "links.txt")]
    input: PathBuf,

    /// Tên và đường dẫn file EPUB đầu ra
    #[arg(
        short = 'o',
        long = "output",
        help = "Đường dẫn file đầu ra [Mặc định: <tiêu_đề_sách>.epub]",
        default_value = "output.epub"
    )]
    output: PathBuf,

    /// Tiêu đề chính của cuốn sách EPUB
    #[arg(short, long, default_value = "doc_nhanh")]
    title: String,

    /// Thời gian chờ (mili-giây) giữa các lần tải để tránh bị chặn (Rate Limit)
    #[arg(short, long, default_value_t = 1500)]
    delay: u64,
}

/// Đại diện cho một bức ảnh được tải về để lưu offline
#[derive(Debug)]
struct ImageItem {
    id: String,       // VD: "img_1_1"
    filename: String, // VD: "img_1_1.jpg"
    data: Vec<u8>,    // Dữ liệu byte nhị phân của ảnh JPEG
}

/// Đại diện cho một chương sách (XHTML)
#[derive(Debug)]
struct Chapter {
    id: String,      // VD: "chapter_1"
    title: String,   // Tiêu đề thực tế của bài viết
    content: String, // Nội dung XHTML đã được lọc sạch
}

/// Toàn bộ dữ liệu của cuốn sách EPUB nằm trong RAM
#[derive(Debug, Default)]
struct EpubBook {
    title: String,
    chapters: Vec<Chapter>,
    images: Vec<ImageItem>,
}

impl EpubBook {
    /// Bước 4.1: Sinh nội dung file cấu trúc chính content.opf
    fn generate_opf(&self) -> String {
        let utc_now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let timestamp = Utc::now().timestamp();

        // Khởi tạo phần đầu Metadata
        let mut opf = format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" unique-identifier="bookid" version="3.0">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>{}</dc:title>
    <dc:identifier id="bookid">urn:uuid:doc-nhanh-compilation-{}</dc:identifier>
    <dc:language>vi</dc:language>
    <meta property="dcterms:modified">{}</meta>
  </metadata>
  <manifest>
    <item id="toc" href="toc.xhtml" media-type="application/xhtml+xml" properties="nav"/>
    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
    <item id="css" href="CSS/template.css" media-type="text/css"/>
"#,
            self.title, timestamp, utc_now
        );

        // Vòng lặp Manifest: Đăng ký toàn bộ các Chương (XHTML)
        for chapter in &self.chapters {
            opf.push_str(&format!(
                "    <item id=\"{}\" href=\"{}.xhtml\" media-type=\"application/xhtml+xml\"/>\n",
                chapter.id, chapter.id
            ));
        }

        // Vòng lặp Manifest: Đăng ký toàn bộ các Ảnh (JPEG) đã tối ưu
        for img in &self.images {
            opf.push_str(&format!(
                "    <item id=\"{}\" href=\"Images/{}\" media-type=\"image/jpeg\"/>\n",
                img.id, img.filename
            ));
        }

        opf.push_str("  </manifest>\n  <spine toc=\"ncx\">\n");

        // Vòng lặp Spine: Thiết lập thứ tự lật trang của các chương
        for chapter in &self.chapters {
            opf.push_str(&format!("    <itemref idref=\"{}\"/>\n", chapter.id));
        }

        opf.push_str("  </spine>\n</package>");
        opf
    }

    /// Bước 4.2a: Sinh file Mục lục chuẩn EPUB 3 (toc.xhtml)
    fn generate_toc_xhtml(&self) -> String {
        let mut toc = r#"<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head>
  <title>Mục lục</title>
  <link href="CSS/template.css" rel="stylesheet" type="text/css" />
</head>
<body>
  <nav epub:type="toc" id="toc">
    <h1>Mục lục</h1>
    <ol>
"#
        .to_string();

        for chapter in &self.chapters {
            toc.push_str(&format!(
                "      <li><a href=\"{}.xhtml\">{}</a></li>\n",
                chapter.id, chapter.title
            ));
        }

        toc.push_str("    </ol>\n  </nav>\n</body>\n</html>");
        toc
    }

    /// Bước 4.2b: Sinh file Mục lục Fallback cho thiết bị cũ (toc.ncx)
    fn generate_toc_ncx(&self) -> String {
        let timestamp = Utc::now().timestamp();
        let mut ncx = format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head>
    <meta name="dtb:uid" content="urn:uuid:doc-nhanh-compilation-{}" />
    <meta name="dtb:depth" content="1" />
    <meta name="dtb:totalPageCount" content="0" />
    <meta name="dtb:maxPageNumber" content="0" />
  </head>
  <docTitle>
    <text>{}</text>
  </docTitle>
  <navMap>
"#,
            timestamp, self.title
        );

        for (idx, chapter) in self.chapters.iter().enumerate() {
            let play_order = idx + 1;
            ncx.push_str(&format!(
                r#"    <navPoint id="{}" playOrder="{}">
      <navLabel>
        <text>{}</text>
      </navLabel>
      <content src="{}.xhtml" />
    </navPoint>
"#,
                chapter.id, play_order, chapter.title, chapter.id
            ));
        }

        ncx.push_str("  </navMap>\n</ncx>");
        ncx
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Khởi tạo đối tượng sách trong bộ nhớ
    let book = EpubBook {
        title: args.title.clone(),
        ..Default::default()
    };

    println!(
        "+ Đã khởi tạo khung sách trống trong RAM: \"{}\"",
        book.title
    );

    // Khởi tạo khung sách trống trong RAM
    let mut book = EpubBook {
        title: args.title.clone(),
        chapters: Vec::new(),
        images: Vec::new(),
    };

    // RẼ NHÁNH XỬ LÝ THEO CHẾ ĐỘ NGƯỜI DÙNG CHỌN
    match args.mode.as_str() {
        "web" => {
            println!("=====================================");
            println!("       PHÂN TÍCH FILE ĐẦU VÀO        ");
            println!("=====================================");

            // Dùng toán tử ? để đẩy lỗi ra ngoài nếu không đọc được file
            let content = std::fs::read_to_string(&args.input).map_err(|e| {
                format!(
                    "Không thể mở file cấu hình '{}': {}",
                    args.input.display(),
                    e
                )
            })?;

            let urls: Vec<String> = content
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .collect();

            println!("+ Tìm thấy {} URL hợp lệ để tiến hành tải.", urls.len());

            println!("=====================================");
            println!("       TIẾN HÀNH CÀO DỮ LIỆU         ");
            println!("=====================================");

            let mut img_counter = 0;

            for (idx, url_str) in urls.iter().enumerate() {
                let chapter_num = idx + 1;
                println!("[{}/{}] Đang tải: {}", chapter_num, urls.len(), url_str);

                // [SỬA LỖI UREQ 3.3]: Đọc trực tiếp ra String từ hàm không tham số
                let html_raw = match ureq::get(url_str)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .call()
    {
        Ok(response) => match response.into_body().read_to_string() {
            Ok(html) => html,
            Err(e) => {
                eprintln!("⚠️  [Bỏ qua] Không thể đọc HTML từ {}: {}", url_str, e);
                continue;
            }
        },
        Err(e) => {
            eprintln!("❌ [Skipped URL do lỗi mạng/HTTP] URL: {} | Chi tiết: {}", url_str, e);
            continue;
        }
    };

                // [SỬA LỖI DOM_SMOOTHIE 0.18]: Truyền đủ 3 tham số (html_raw được move vào luôn), unwrap Result trước khi parse
                let mut readability =
                    match dom_smoothie::Readability::new(html_raw, Some(url_str.as_str()), None) {
                        Ok(r) => r,
                        Err(_) => {
                            eprintln!(
                                "  [Cảnh báo] Lỗi khởi tạo bộ lọc Readability cho trang này."
                            );
                            continue;
                        }
                    };

                let article = match readability.parse() {
                    Ok(art) => art,
                    Err(_) => {
                        eprintln!("  [Cảnh báo] Không thể trích xuất nội dung chính từ trang này.");
                        continue;
                    }
                };

                let mut content_html = article.content.to_string();

                let base_url = match Url::parse(url_str) {
                    Ok(u) => u,
                    Err(_) => {
                        eprintln!("  [Cảnh báo] Lỗi định dạng Base URL: {}", url_str);
                        continue;
                    }
                };

                // Xử lý Ảnh (Image Pipeline)
                let document = Html::parse_document(&content_html);
                let img_selector = Selector::parse("img")?;

                let mut img_replacements = Vec::new();

                for img_element in document.select(&img_selector) {
                    if let Some(src) = img_element.value().attr("src") {
                        let absolute_img_url = match base_url.join(src) {
                            Ok(u) => u,
                            Err(_) => continue,
                        };

                        // Tải bytes của ảnh
                        let img_bytes = match ureq::get(absolute_img_url.as_str()).call() {
                            Ok(resp) => resp.into_body().read_to_vec().unwrap_or_default(),
                            Err(_) => {
                                eprintln!(
                                    "   └── ⚠️  [Cảnh báo] Không tải được ảnh tại: {}",
                                    absolute_img_url
                                );
                                continue; // Chỉ bỏ qua bức ảnh lỗi này, vòng lặp ảnh vẫn chạy tiếp
                            }
                        };

                        if img_bytes.is_empty() {
                            continue;
                        }

                        // Xử lý giải mã ảnh trong RAM
                        let dynamic_img = match image::load_from_memory(&img_bytes) {
                            Ok(img) => img,
                            Err(_) => {
                                eprintln!(
                                    "   └── ⚠️  [Cảnh báo] Định dạng ảnh không hợp lệ tại: {}",
                                    absolute_img_url
                                );
                                continue; // Ảnh hỏng/gif không hỗ trợ -> bỏ qua ảnh
                            }
                        };

                        let mut jpeg_bytes = Vec::new();

                        if dynamic_img
                            .write_to(
                                &mut std::io::Cursor::new(&mut jpeg_bytes),
                                image::ImageFormat::Jpeg,
                            )
                            .is_ok()
                        {
                            img_counter += 1;
                            let img_id = format!("img_{}_{}", chapter_num, img_counter);
                            let filename = format!("{}.jpg", img_id);

                            book.images.push(ImageItem {
                                id: img_id,
                                filename: filename.clone(),
                                data: jpeg_bytes,
                            });

                            img_replacements
                                .push((src.to_string(), format!("Images/{}", filename)));
                        }
                    }
                }

                for (old_src, new_src) in img_replacements {
                    content_html = content_html.replace(&old_src, &new_src);
                }

                // Bọc nội dung vào template XHTML chuẩn của EPUB 3
                let xhtml_content = format!(
                    r#"<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head>
<title>{}</title>
<link href="CSS/template.css" rel="stylesheet" type="text/css" />
</head>
<body>
    <h1>{}</h1>
    {}
</body>
</html>"#,
                    article.title, article.title, content_html
                );

                // Đưa chương hoàn chỉnh vào sách
                book.chapters.push(Chapter {
                    id: format!("chapter_{}", chapter_num),
                    title: article.title.to_string(),
                    content: xhtml_content,
                });

                println!("  => [OK] Đã xử lý xong chương: {}", article.title);

                if idx < urls.len() - 1 {
                    sleep(Duration::from_millis(args.delay));
                }
            }

            println!("=====================================");
            println!(
                "+ Hoàn tất tải: {}/{} bài viết thành công.",
                book.chapters.len(),
                urls.len()
            );
            println!(
                "+ Tổng dung lượng ảnh offline đã tối ưu: {} file ảnh.",
                book.images.len()
            );

            println!("=====================================\n");

            // KIỂM TRA ĐIỀU KIỆN BIÊN: Nếu không có chương nào thành công, dừng chương trình luôn
            if book.chapters.is_empty() {
                println!("=================================================");
                println!("❌ HỦY QUY TRÌNH: Không có bài viết nào tải thành công!");
                println!("👉 Hệ thống dừng lại để tránh tạo tệp EPUB rỗng vô nghĩa.");
                println!("=================================================");
                return Ok(()); // Thoát hàm main một cách an toàn
            }
        }

        "md" => {
            println!("=====================================");
            println!("      PHÂN TÍCH THƯ MỤC MARKDOWN     ");
            println!("=====================================");

            // Đọc danh sách file trong thư mục, lỗi ở đây là lỗi Fatal hệ thống nên dùng toán tử ?
            let entries = std::fs::read_dir(&args.input)
                .map_err(|e| format!("Không thể mở thư mục '{}': {}", args.input.display(), e))?;

            let mut md_files: Vec<_> = entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
                .collect();

            // Sắp xếp file theo tên (A-Z) để đảm bảo đúng thứ tự chương: 01_intro.md, 02_chap1.md...
            md_files.sort();

            if md_files.is_empty() {
                return Err(format!(
                    "Thư mục '{}' trống hoặc không chứa file .md nào!",
                    args.input.display()
                )
                .into());
            }

            println!(
                "+ Tìm thấy {} file Markdown hợp lệ để biên dịch.",
                md_files.len()
            );
            println!("=====================================");
            println!("       TIẾN HÀNH BIÊN DỊCH CHƯƠNG    ");
            println!("=====================================");

            for (index, path) in md_files.iter().enumerate() {
                let chapter_num = index + 1;
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();

                println!(
                    "[{}/{}] Đang xử lý: {}",
                    chapter_num,
                    md_files.len(),
                    file_name
                );

                // Đọc nội dung file .md (Áp dụng luật Giai đoạn 6: file lỗi cục bộ thì bỏ qua)
                let md_content = match std::fs::read_to_string(path) {
                    Ok(content) => content,
                    Err(e) => {
                        eprintln!(
                            "  └── ⚠️ [Bỏ qua] Lỗi không thể đọc file '{}': {}",
                            file_name, e
                        );
                        continue;
                    }
                };

                // DỊCH MARKDOWN SANG HTML BẰNG PULLDOWN-CMARK TRONG RAM
                let mut html_output = String::new();
                let parser = pulldown_cmark::Parser::new(&md_content);
                pulldown_cmark::html::push_html(&mut html_output, parser);

                // Chuẩn hóa tên file làm tiêu đề chương (Ví dụ: "01_gioi_thieu" -> "01 gioi thieu")
                let file_stem = path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let title_clean = file_stem.replace(['_', '-'], " ");

                // Bọc vào khuôn mẫu XHTML tiêu chuẩn để tương thích với trình đọc sách
                let xhtml_content = format!(
                    r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{}</title>
    <link rel="stylesheet" type="text/css" href="CSS/template.css"/>
</head>
<body>
    <h1>{}</h1>
    {}
</body>
</html>"#,
                    title_clean, title_clean, html_output
                );

                // Đổ thẳng vào thùng chứa dữ liệu dùng chung
                book.chapters.push(Chapter {
                    id: format!("chapter_{}", chapter_num),
                    title: title_clean,
                    content: xhtml_content,
                });
            }
        }

        _ => {
            return Err(format!(
                "Chế độ '--mode {}' không hợp lệ! Chỉ chấp nhận 'web' hoặc 'md'.",
                args.mode
            )
            .into());
        }
    }

    // --- KIỂM TRA ĐIỀU KIỆN BIÊN (Chặn tạo sách rỗng như đã cấu hình trước đó) ---
    println!(
        "+ Hoàn tất thu thập: {} bài viết/chương thành công.",
        book.chapters.len()
    );
    if book.chapters.is_empty() {
        println!("=================================================");
        println!("❌ HỦY QUY TRÌNH: Không có nội dung nào được tạo lập!");
        println!("=================================================");
        return Ok(());
    }

    println!("=====================================");
    println!("       SINH METADATA TRONG RAM       ");
    println!("=====================================");

    let opf_content = book.generate_opf();
    let toc_xhtml = book.generate_toc_xhtml();
    let toc_ncx = book.generate_toc_ncx();

    println!(
        "+ [OK] Đã cấu trúc xong tệp 'content.opf' ({}$ bytes)",
        opf_content.len()
    );
    println!(
        "+ [OK] Đã lập bản đồ mục lục 'toc.xhtml' ({}$ bytes)",
        toc_xhtml.len()
    );
    println!(
        "+ [OK] Đã làm tương thích ngược 'toc.ncx' ({}$ bytes)",
        toc_ncx.len()
    );
    println!("=====================================\n");
    println!("=====================================");
    println!("       TIẾN HÀNH ĐÓNG GÓI EPUB       ");
    println!("=====================================");

    // Bước 5.1: Khởi tạo file .epub vật lý và bọc trong ZipWriter
    let epub_path = &args.output;
    let file = File::create(epub_path).map_err(|e| {
        format!(
            "Lỗi hệ thống: Không thể tạo file thành phẩm '{:?}': {}",
            epub_path, e
        )
    })?;

    let mut zip = zip::ZipWriter::new(file);

    // Bước 5.2: Ghi tệp `mimetype` (BẮT BUỘC KHÔNG NÉN - Stored)
    let options_stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    zip.start_file("mimetype", options_stored)?;
    zip.write_all(b"application/epub+zip")?;

    // Bước 5.3: Đổi thuật toán sang NÉN (Deflated) cho toàn bộ các file sau
    let options_deflated =
        SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    // Bước 5.4: Ghi cấu trúc thư mục tĩnh & Metadata
    // 1. Ghi file META-INF/container.xml
    let container_xml = r#"<?xml version="1.0" encoding="utf-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#;

    zip.start_file("META-INF/container.xml", options_deflated)?;
    zip.write_all(container_xml.as_bytes())?;

    // 2. Ghi file OEBPS/content.opf (Đã sinh ở Giai đoạn 4)
    zip.start_file("OEBPS/content.opf", options_deflated)?;
    zip.write_all(opf_content.as_bytes())?;

    // 3. Ghi file OEBPS/toc.xhtml và toc.ncx (Đã sinh ở Giai đoạn 4)
    zip.start_file("OEBPS/toc.xhtml", options_deflated)?;
    zip.write_all(toc_xhtml.as_bytes())?;

    zip.start_file("OEBPS/toc.ncx", options_deflated)?;
    zip.write_all(toc_ncx.as_bytes())?;

    // 4. Ghi file OEBPS/CSS/template.css (Hardcode style hiển thị đẹp mắt)
    let css_content = r#"
body {
    font-family: "Helvetica Neue", Helvetica, Arial, sans-serif;
    margin: 5%, 5%, 5%, 5%;
    line-height: 1.6;
    color: #111111;
}
h1 {
    text-align: center;
    color: #2c3e50;
    font-size: 1.8em;
    margin-top: 1em;
    margin-bottom: 1.5em;
    border-bottom: 2px solid #ecf0f1;
    padding-bottom: 0.5em;
}
p {
    text-indent: 1.5em;
    margin-bottom: 0.8em;
    text-align: justify;
}
img {
    max-width: 100%;
    height: auto;
    display: block;
    margin: 1.5em auto;
    border-radius: 4px;
    box-shadow: 0 2px 5px rgba(0,0,0,0.15);
}
"#;
    zip.start_file("OEBPS/CSS/template.css", options_deflated)?;
    zip.write_all(css_content.as_bytes())?;
    println!("+ [OK] Đã khởi tạo xong các tệp cấu trúc hệ thống & CSS.");

    // Bước 5.5: Ghi các Chương sách & Ảnh từ bộ nhớ RAM xuống
    // Ghi các chương bài viết (XHTML)
    for chapter in &book.chapters {
        let file_path = format!("OEBPS/{}.xhtml", chapter.id);
        zip.start_file(&file_path, options_deflated)?;
        zip.write_all(chapter.content.as_bytes())?;
    }
    println!(
        "+ [OK] Đã nén và đóng gói xong {} chương nội dung.",
        book.chapters.len()
    );

    // Ghi các tệp ảnh nhị phân (JPEG raw bytes)
    for img in &book.images {
        let file_path = format!("OEBPS/Images/{}", img.filename);
        zip.start_file(&file_path, options_deflated)?;
        zip.write_all(&img.data)?;
    }
    if !book.images.is_empty() {
        println!(
            "+ [OK] Đã đóng gói toàn bộ {} tệp ảnh offline.",
            book.images.len()
        );
    }

    // Bước 5.6: Kết thúc và chốt hạ tệp
    zip.finish()
        .map_err(|e| format!("Không thể hoàn tất đóng gói ZIP: {}", e))?;

    Ok(())
}

// =================================================================
// TẦNG 1: UNIT TESTS - KIỂM THỬ LOGIC NỘI BỘ TRONG RAM
// =================================================================
#[cfg(test)]
mod tests {
    use super::*; // Kế thừa toàn bộ Struct và Hàm từ main.rs lên đây để test
    use url::Url;

    /// Hàm helper để khởi tạo nhanh một đối tượng EpubBook giả lập (Mock) phục vụ test
    fn create_mock_book() -> EpubBook {
        EpubBook {
            title: "Sách Test Tự Động".to_string(),
            chapters: vec![
                Chapter {
                    id: "chapter_1".to_string(),
                    title: "Chương 1: Khởi Đầu".to_string(),
                    content: "<p>Nội dung chương 1</p>".to_string(),
                },
                Chapter {
                    id: "chapter_2".to_string(),
                    title: "Chương 2: Tăng Tốc".to_string(),
                    content: "<p>Nội dung chương 2</p>".to_string(),
                },
            ],
            images: vec![ImageItem {
                id: "img_1_1".to_string(),
                filename: "img_1_1.jpg".to_string(),
                data: vec![0, 1, 2, 3], // Giả lập vài bytes dữ liệu ảnh raw
            }],
        }
    }

    #[test]
    fn test_generate_opf_structure() {
        let book = create_mock_book();
        let opf_result = book.generate_opf();

        // 1. Kiểm tra tiêu đề sách được nhúng chính xác vào metadata
        assert!(opf_result.contains("<dc:title>Sách Test Tự Động</dc:title>"));

        // 2. Kiểm tra xem các chương đã được khai báo đầy đủ trong <manifest> chưa
        assert!(opf_result.contains(
            "<item id=\"chapter_1\" href=\"chapter_1.xhtml\" media-type=\"application/xhtml+xml\"/>"
        ));
        assert!(opf_result.contains(
            "<item id=\"chapter_2\" href=\"chapter_2.xhtml\" media-type=\"application/xhtml+xml\"/>"
        ));

        // 3. Kiểm tra xem ảnh offline đã được đăng ký định dạng jpeg chưa
        assert!(opf_result.contains(
            "<item id=\"img_1_1\" href=\"Images/img_1_1.jpg\" media-type=\"image/jpeg\"/>"
        ));

        // 4. Kiểm tra luồng lật trang <spine> có xếp đúng thứ tự từ chương 1 đến chương 2 không
        assert!(opf_result.contains("<itemref idref=\"chapter_1\"/>"));
        assert!(opf_result.contains("<itemref idref=\"chapter_2\"/>"));
    }

    #[test]
    fn test_generate_toc_xhtml_epub3() {
        let book = create_mock_book();
        let toc_xhtml = book.generate_toc_xhtml();

        // 1. Đảm bảo file mục lục chứa định dạng điều hướng bắt buộc của EPUB 3 (`epub:type="toc"`)
        assert!(toc_xhtml.contains("epub:type=\"toc\""));

        // 2. Kiểm tra các liên kết lật chương có ánh xạ đúng tiêu đề không
        assert!(toc_xhtml.contains("<a href=\"chapter_1.xhtml\">Chương 1: Khởi Đầu</a>"));
        assert!(toc_xhtml.contains("<a href=\"chapter_2.xhtml\">Chương 2: Tăng Tốc</a>"));
    }

    #[test]
    fn test_url_absolute_resolution_logic() {
        // Giả lập tình huống xử lý đường dẫn ảnh tương đối trên các trang web thật
        let base_url = Url::parse("https://blog.example.com/posts/kinh-nghiem-code.html").unwrap();

        // Tình huống 1: Đường dẫn ảnh tương đối cùng cấp hoặc lùi cấp
        let relative_src_1 = "../assets/banner.jpg";
        let resolved_1 = base_url.join(relative_src_1).unwrap();
        assert_eq!(
            resolved_1.as_str(),
            "https://blog.example.com/assets/banner.jpg"
        );

        // Tình huống 2: Đường dẫn ảnh bắt đầu bằng dấu gạch chéo căn (root-relative)
        let relative_src_2 = "/images/avatar.png";
        let resolved_2 = base_url.join(relative_src_2).unwrap();
        assert_eq!(
            resolved_2.as_str(),
            "https://blog.example.com/images/avatar.png"
        );
    }

    #[test]
    #[should_panic]
    fn test_invalid_base_url_handling() {
        // Test xem hệ thống core của thư viện url có crash chủ động nếu gặp url rác hay không
        // (Thuộc tính #[should_panic] kỳ vọng đoạn code này PHẢI sụp đổ thì test mới tính là ĐẠT)
        Url::parse("chuo_i_rac_khong_phai_url").unwrap();
    }
}
