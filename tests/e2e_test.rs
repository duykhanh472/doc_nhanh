use httpmock::prelude::*;
use std::fs::File;
use std::io::{Read, Write};
use std::process::Command;
use tempfile::NamedTempFile;
use zip::ZipArchive;

#[test]
fn test_e2e_epub_output_integrity() {
    // 1. DỰNG INTERNET GIẢ LẬP ĐỂ CÀO DỮ LIỆU THẬT
    let server = MockServer::start();

    let html_content_1 = "<html><body><h1>Chuong Một</h1><p>Nội dung 1</p></body></html>";
    let html_content_2 = "<html><body><h1>Chuong Hai</h1><p>Nội dung 2</p></body></html>";

    let mock_1 = server.mock(|when, then| {
        when.method(GET).path("/page1");
        then.status(200).body(html_content_1);
    });
    let mock_2 = server.mock(|when, then| {
        when.method(GET).path("/page2");
        then.status(200).body(html_content_2);
    });

    // Tạo file test_links.txt tạm thời chứa 2 link giả lập
    let mut temp_links_file = NamedTempFile::new().expect("Không thể tạo file links tạm");
    writeln!(temp_links_file, "{}", server.url("/page1")).unwrap();
    writeln!(temp_links_file, "{}", server.url("/page2")).unwrap();

    let links_path = temp_links_file.path().to_str().unwrap();
    let output_epub_path = "target/debug/e2e_thanh_pham.epub";

    // 2. CHẠY PHẦN MỀM QUA CLI (Giao diện dòng lệnh)
    let cli_status = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "--input",
            links_path,
            "--title",
            "Sách E2E Đạt Chuẩn",
            "--output",
            output_epub_path,
        ])
        .status()
        .expect("Không thể kích hoạt lệnh cargo run");

    // Khẳng định chương trình phải kết thúc với mã thoát thành công (0)
    assert!(cli_status.success(), "Ứng dụng CLI bị crash giữa chừng!");

    // 3. MỔ XẺ FILE .EPUB VÀ KIỂM TRA NỘI THẤT (Unzip Inspection)
    let epub_file =
        File::open(output_epub_path).expect("File .epub thành phẩm không tồn tại trên ổ cứng!");

    // Dùng ZipArchive để đọc cấu trúc file nén trực tiếp trong bộ nhớ
    let mut archive =
        ZipArchive::new(epub_file).expect("Định dạng file không phải là tệp ZIP tiêu chuẩn!");

    // Kiểm tra trạm gác 1: File 'mimetype' bắt buộc phải nằm ĐẦU TIÊN và KHÔNG ĐƯỢC NÉN
    {
        // Thừa hành tiêu chuẩn EPUB: file đầu tiên tại index 0 phải là mimetype
        let mut mimetype_file = archive
            .by_index(0)
            .expect("Không tìm thấy tệp tin đầu tiên trong cấu trúc gói!");

        assert_eq!(
            mimetype_file.name(),
            "mimetype",
            "Tệp đầu tiên của sách phải có tên là 'mimetype'!"
        );

        // Đảm bảo thuật toán nén là Stored (không nén - size gốc bằng size sau nén)
        assert_eq!(
            mimetype_file.compression(),
            zip::CompressionMethod::Stored,
            "Lỗi nghiêm trọng: Tệp mimetype đang bị nén! Trình đọc sách sẽ từ chối mở."
        );

        // Kiểm tra nội dung chuỗi bên trong
        let mut content = String::new();
        mimetype_file.read_to_string(&mut content).unwrap();
        assert_eq!(
            content, "application/epub+zip",
            "Nội dung tệp mimetype bị sai cấu trúc định dạng!"
        );
    }

    // Kiểm tra trạm gác 2: Định vị file cấu trúc lõi 'OEBPS/content.opf'
    {
        let opf_file = archive.by_name("OEBPS/content.opf");
        assert!(
            opf_file.is_ok(),
            "Thiếu tệp điều phối hệ thống 'OEBPS/content.opf'!"
        );

        let mut opf_content = String::new();
        opf_file.unwrap().read_to_string(&mut opf_content).unwrap();

        // Đảm bảo metadata tiêu đề người dùng nhập từ CLI được ghi nhận chính xác vào file opf
        assert!(opf_content.contains("<dc:title>Sách E2E Đạt Chuẩn</dc:title>"));
    }

    // Kiểm tra trạm gác 3: Định vị tệp container điều hướng bắt buộc
    assert!(
        archive.by_name("META-INF/container.xml").is_ok(),
        "Thiếu tệp cấu trúc định tuyến 'META-INF/container.xml'!"
    );

    // Kiểm tra trạm gác 4: Xem các file chương sách có được đặt đúng chỗ không
    assert!(archive.by_name("OEBPS/chapter_1.xhtml").is_ok());
    assert!(archive.by_name("OEBPS/chapter_2.xhtml").is_ok());

    // 4. DỌN DẸP CHIẾN TRƯỜNG
    mock_1.assert_hits(1);
    mock_2.assert_hits(1);
    let _ = std::fs::remove_file(output_epub_path);
}
