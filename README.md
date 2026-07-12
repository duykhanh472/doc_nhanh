<a id="readme-top"></a>

<div align="center">

[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stars][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]

</div>

<br />
<div align="center">
  <a href="https://github.com/duykhanh472/doc_nhanh">
    <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Logo" width="120">
  </a>

<h3 align="center">doc_nhanh</h3>

  <p align="center">
    Công cụ CLI bằng Rust chuyển đổi Web URLs hoặc thư mục chứa các tệp Markdown thành sách điện tử EPUB
    <br />
    <a href="https://github.com/duykhanh472/doc_nhanh"><strong>Đọc tài liệu »</strong></a>
    <br />
    <br />
    <a href="https://github.com/duykhanh472/doc_nhanh">Xem Demo</a>
    &middot;
    <a href="https://github.com/duykhanh472/doc_nhanh/issues">Báo lỗi</a>
    &middot;
    <a href="https://github.com/duykhanh472/doc_nhanh/issues">Yêu cầu tính năng</a>
  </p>
</div>



<details>
  <summary>Mục lục dự án</summary>
  <ol>
    <li>
      <a href="#about-the-project">Về dự án</a>
      <ul>
        <li><a href="#built-with">Công nghệ cốt lõi</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Cài đặt ban đầu</a>
      <ul>
        <li><a href="#prerequisites">Điều kiện tiên quyết</a></li>
        <li><a href="#installation">Cài đặt chi tiết</a></li>
      </ul>
    </li>
    <li><a href="#usage">Hướng dẫn sử dụng</a></li>
    <li><a href="#roadmap">Lộ trình phát triển</a></li>
    <li><a href="#contributing">Đóng góp mã nguồn</a></li>
    <li><a href="#license">Giấy phép</a></li>
    <li><a href="#contact">Liên hệ</a></li>
    <li><a href="#acknowledgments">Lời cảm ơn</a></li>
  </ol>
</details>

## About The Project

**doc_nhanh** ra đời nhằm giải quyết nhu cầu đóng gói tài liệu tự động của các lập trình viên và độc giả yêu thích đọc sách offline trên Kindle hoặc các ứng dụng e-reader chuyên dụng. 

### Các tính năng vượt trội:

- **Chế độ Kép linh hoạt:** Hỗ trợ tải bài viết trực tiếp từ danh sách URL trang web (`web`) hoặc biên dịch hàng loạt tệp `.md` cục bộ (`md`) thành các chương sách.
- **Cơ chế Chịu tải lỗi cao (Fault-tolerance):** Tự động bỏ qua các liên kết lỗi mạng (`404`, `500`), các tệp lỗi phân quyền, hoặc ảnh hỏng để tiếp tục tiến trình đóng gói sách thay vì sụp đổ (crash) chương trình.
- **Bypass Cơ chế Chống Bot:** Tích hợp tùy biến định danh trình duyệt `User-Agent` chuẩn Chrome giúp vượt qua hàng rào chặn mã lỗi `403 Forbidden` của các hệ thống lớn như Fandom Wiki, Cloudflare.
- **Chống ghi đè thông minh:** Tên tệp đầu ra được đính kèm tự động mốc thời gian thực máy tính `Năm-Tháng-Ngày_Giờ-Phút` (Ví dụ: `output_2026-07-12_14-30.epub`), tương thích an toàn trên cả hệ điều hành Windows mà không lo trùng lặp.
- **Kiểm soát chất lượng bằng Tháp 3 Tầng:** Tích hợp Unit Test logic RAM, Integration Test với Server mạng giả lập (`httpmock`), và End-to-End Test mổ xẻ cấu trúc ZIP của file `.epub` đầu ra để đảm bảo file thành phẩm luôn đạt chuẩn.

<p align="right">(<a href="#readme-top">về đầu trang</a>)</p>



### Built With

Sản phẩm được xây dựng hoàn toàn bằng ngôn ngữ Rust nguyên bản cho hiệu năng cực hạn và không phụ thuộc vào các công cụ bên ngoài hệ thống (như Pandoc).

<p align="right">(<a href="#readme-top">về đầu trang</a>)</p>


## Getting Started

Để tải về, biên dịch và chạy thử hệ thống kiểm thử của dự án trên máy tính cục bộ của bạn, hãy làm theo các bước đơn giản sau:

### Prerequisites

Máy tính của bạn cần cài đặt sẵn môi trường biên dịch Rust (Rust toolchain):
- Rust & Cargo (Phiên bản mới nhất khuyên dùng)
  ```sh
  curl --proto '=https' --tlsv1.2 -sSf [https://sh.rustup.rs](https://sh.rustup.rs) | sh

  ```

### Installation

1. Clone mã nguồn của dự án từ GitHub
```sh
git clone [https://github.com/duykhanh472/doc_nhanh.git](https://github.com/duykhanh472/doc_nhanh.git)

```


2. Di chuyển vào thư mục dự án
```sh
cd doc_nhanh

```


3. Kiểm tra và chạy thử toàn bộ Tháp kiểm thử tự động (Unit + Integration + E2E) để đảm bảo môi trường tương thích hoàn toàn:
```sh
cargo test

```


4. Biên dịch phiên bản tối ưu hóa hiệu năng cao (Release mode)
```sh
cargo build --release

```



## Usage

Ứng dụng hỗ trợ giao diện dòng lệnh (CLI) cực kỳ tường minh. Bạn có thể tương tác với 2 chế độ độc lập:

### Chế độ 1: Tải dữ liệu trực tuyến (Mode Web)

Tạo một tệp `links.txt` chứa danh sách các URL cần tải (mỗi dòng một URL), sau đó chạy lệnh:

```sh
cargo run -- --mode web --input links.txt --title "Cẩm Nang Lập Trình Rust"

```

*Hệ thống sẽ tự động tải bài viết, tối ưu ảnh offline và lưu tệp thành `2026-07-12_14-45.epub`.*

### Chế độ 2: Biên dịch thư mục Markdown sẵn có (Mode MD)

Chuẩn bị một thư mục chứa các file bài viết định dạng `.md` (Ví dụ: `01_intro.md`, `02_chap1.md`), sau đó gõ lệnh:

```sh
cargo run -- --mode md --input ./my_chapters/ --title "Truyện Tôi Viết" --output truyen_hay.epub

```

*Ứng dụng sẽ tự động sắp xếp tên file theo thứ tự ký tự (A-Z), chuyển dịch cú pháp Markdown sang XHTML chuẩn EPUB 3, nén không mất dữ liệu cấu trúc `mimetype` ở index 0.*

## Roadmap

- [x] Phát triển bộ lọc nội dung lõi (Readability Pipeline) lọc bỏ Script/Ads rác.
- [x] Phát triển cơ chế tải và nhúng ảnh cục bộ sang Base64/Images Folder.
- [x] Tích hợp bộ xử lý lỗi mạng thông minh (Bypass 403 / Skip Link Die 404).
- [x] Hỗ trợ tính năng biên dịch file Markdown cục bộ.
- [x] Tự động sinh tên file theo mốc thời gian thực tế chống ghi đè.
- [ ] Tính năng tự động tạo ảnh bìa (Cover Image) từ tiêu đề sách.
- [ ] Hỗ trợ đa luồng (Multi-threading) tăng tốc tải bài viết đồng thời.

Xem danh sách [open issues](https://github.com/duykhanh472/doc_nhanh/issues) để cập nhật và đề xuất các tính năng mới cho cộng đồng.

## Contributing

Mọi đóng góp mã nguồn nhằm tối ưu hóa hiệu năng hoặc cải tiến định dạng cấu trúc CSS cho sách thành phẩm của cộng đồng đều **được trân quý ghi nhận**.

1. Fork dự án
2. Tạo nhánh Feature mới (`git checkout -b feature/AmazingFeature`)
3. Commit những thay đổi của bạn (`git commit -m 'Add some AmazingFeature'`)
4. Đẩy nhánh Feature lên GitHub (`git push origin feature/AmazingFeature`)
5. Mở một Pull Request đối chiếu


## License

Dự án này được phát hành dưới giấy phép **CC0 1.0 Universal (Public Domain)**. 

Điều này có nghĩa là bạn có quyền tự do:

* **Sao chép và phân phối:** Bạn có thể sao chép và phân phối bản sao của dự án dưới mọi hình thức.
* **Sửa đổi và kết hợp:** Bạn có quyền chỉnh sửa, thay đổi và sử dụng mã nguồn cho bất kỳ mục đích nào (kể cả thương mại) mà không cần xin phép.
* **Không yêu cầu ghi công:** Mặc dù việc ghi công (attribution) là một hành động đẹp trong cộng đồng mã nguồn mở, nhưng theo giấy phép CC0, bạn không bắt buộc phải thực hiện điều đó.

Để biết thêm chi tiết, vui lòng xem nội dung đầy đủ của [CC0 1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/legalcode).

<p align="right">(<a href="#readme-top">về đầu trang</a>)</p>

## Acknowledgments

- [IDPF (International Digital Publishing Forum)](https://idpf.org/) - Tổ chức định hình và kiểm duyệt tiêu chuẩn tệp tin sách điện tử EPUB.
- Cộng đồng Rust mã nguồn mở vì các thư viện phân tích cấu trúc tuyệt vời.


<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->

[contributors-shield]: https://img.shields.io/github/contributors/duykhanh472/doc_nhanh.svg?style=for-the-badge
[contributors-url]: https://github.com/duykhanh472/doc_nhanh/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/duykhanh472/doc_nhanh.svg?style=for-the-badge
[forks-url]: https://github.com/duykhanh472/doc_nhanh/network/members
[stars-shield]: https://img.shields.io/github/stars/duykhanh472/doc_nhanh.svg?style=for-the-badge
[stars-url]: https://github.com/duykhanh472/doc_nhanh/stargazers
[issues-shield]: https://img.shields.io/github/issues/duykhanh472/doc_nhanh.svg?style=for-the-badge
[issues-url]: https://github.com/duykhanh472/doc_nhanh/issues
[license-shield]: https://img.shields.io/badge/License-CC0_1.0-lightgrey.svg?style=for-the-badge
[license-url]: https://creativecommons.org/publicdomain/zero/1.0/
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=555
[linkedin-url]: https://linkedin.com/in/linkedin_username
[product-screenshot]: images/screenshot.png