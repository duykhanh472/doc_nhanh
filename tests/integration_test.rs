use httpmock::prelude::*;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

#[test]
fn test_full_pipeline_with_network_errors() {
    // 1. KHỞI TẠO MOCK SERVER (Giả lập internet cục bộ trong RAM)
    let server = MockServer::start();

    // Kịch bản 1: Bài viết chuẩn nhưng có kèm "thẻ rác" (Kiểm thử bộ lọc Readability)
    let html_with_junk = r#"
        <!DOCTYPE html>
        <html>
        <head><title>Trang Test</title></head>
        <body>
            <div class="quang-cao" style="color:red">Quảng cáo lừa đảo click vào đây!</div>
            <script>alert("Hacker");</script>
            <main>
                <h1>Bài Viết Chất Lượng Cao</h1>
                <p>Đây là nội dung cốt lõi của bài viết mà bộ lọc phải giữ lại.</p>
            </main>
        </body>
        </html>
    "#;
    let mock_good_page = server.mock(|when, then| {
        when.method(GET).path("/chuong-thanh-cong");
        then.status(200)
            .header("content-type", "text/html; charset=utf-8")
            .body(html_with_junk);
    });

    // Kịch bản 2: Đường dẫn chết - Lỗi 404 (Kiểm thử Error Handling bỏ qua link die)
    let mock_404_page = server.mock(|when, then| {
        when.method(GET).path("/bai-viet-bi-xoa-404");
        then.status(404);
    });

    // Kịch bản 3: Bài viết sống nhưng chứa ảnh bị lỗi hệ thống 500 (Kiểm thử bỏ qua ảnh hỏng)
    let html_with_bad_img = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <body>
            <h1>Chương có ảnh hỏng</h1>
            <p>Nội dung chữ vẫn đọc tốt.</p>
            <img src="{}">
        </body>
        </html>
        "#,
        server.url("/anh-bi-loi-500.jpg")
    );
    let mock_bad_img_page = server.mock(|when, then| {
        when.method(GET).path("/chuong-chua-anh-loi");
        then.status(200)
            .header("content-type", "text/html")
            .body(html_with_bad_img);
    });

    let mock_500_image = server.mock(|when, then| {
        when.method(GET).path("/anh-bi-loi-500.jpg");
        then.status(500);
    });

    // 2. TẠO FILE LINKS TẠM THỜI (Nhồi cả link tốt lẫn link lỗi vào luồng chạy)
    let mut temp_links_file = NamedTempFile::new().expect("Không thể tạo file link tạm");
    writeln!(temp_links_file, "{}", server.url("/chuong-thanh-cong")).unwrap();
    writeln!(temp_links_file, "{}", server.url("/bai-viet-bi-xoa-404")).unwrap(); // Gặp 404 -> Phải continue
    writeln!(temp_links_file, "{}", server.url("/chuong-chua-anh-loi")).unwrap(); // Gặp ảnh 500 -> Phải continue ảnh

    let links_path = temp_links_file.path().to_str().unwrap();
    let output_epub = "target/debug/integration_test_result.epub";

    // 3. KÍCH HOẠT LỆNH CLI ĐỂ BLACK-BOX TESTING
    // Chạy trực tiếp app bằng cách gọi `cargo run --` truyền tham số vào
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "--input",
            links_path,
            "--title",
            "Sách Kiểm Thử Tích Hợp",
            "--output",
            output_epub,
        ])
        .output()
        .expect("Thất bại khi ra lệnh thực thi chương trình chính");

    // 4. TIẾN HÀNH ĐỐI CHIẾU KẾT QUẢ (Asserts)
    // Trạng thái thoát của app PHẢI là THÀNH CÔNG (Exit Code = 0).
    // Nếu app bị panic hoặc crash giữa chừng do lỗi 404 hay ảnh hỏng, status.success() sẽ trả về false.
    assert!(
        output.status.success(),
        "Cảnh báo: Chương trình bị sập! Log lỗi terminal:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Kiểm tra xem file sách thành phẩm vật lý `.epub` có được sinh ra thành công hay không
    let file_metadata = std::fs::metadata(output_epub);
    assert!(
        file_metadata.is_ok(),
        "Lỗi: Chương trình thoát thành công nhưng không sinh ra file .epub vật lý!"
    );

    // Đảm bảo Mock Server đã nhận được các lượt ghé thăm từ ứng dụng
    mock_good_page.assert_hits(1);
    mock_404_page.assert_hits(1);
    mock_bad_img_page.assert_hits(1);
    mock_500_image.assert_hits(1);

    // Dọn dẹp file epub rác sau khi test đạt yêu cầu
    let _ = std::fs::remove_file(output_epub);
}
