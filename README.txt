# AIDE Core – Executional Identity Runtime (FIID 3.4)

`aide-core` là thư viện Rust nhỏ gọn hiện thực lõi **Executional Identity** cho Era 5:

- `CryptoHeader` (FIID 3.4) – header 6 byte mô tả thuật toán hook/payload và ZE version.
- **HKDF gate** – tách key cho hook/payload theo từng purpose, tránh key reuse.
- **DIM engine** – hợp nhất các overlay chính sách/thời gian/vị trí với cơ chế pruning.
- **FIID protocol router** – parse QID/QIA/QED và build SUL entry cho AIDE.
- **SRID Registry** – ánh xạ alias SRID (vd: `0xDA12`) sang descriptor đầy đủ.

Thư viện này là **prototype/experimental**, dùng để thử nghiệm các khái niệm FIID 3.4, SRME, SDM và AIDE trong hệ Era 5.

---

## Tính năng chính

### 1. CryptoHeader (FIID 3.4)

- Cấu trúc header 6 byte:

  ```rust
  pub struct CryptoHeader {
      pub version: u8,
      pub hook_alg: u8,
      pub payload_alg: u8,
      pub ze_type: u8,
      pub ze_version: u8,
      pub flags: u8,
  }
  ```

- Hỗ trợ:

  - `encode()` → `[u8; 6]`
  - `decode(&[u8]) -> Result<CryptoHeader, AideError>`

- Validate độ dài header, trả lỗi `AideError::InvalidHeaderLength` nếu không đủ 6 byte.

### 2. HKDF key isolation

- Hàm:

  ```rust
  pub fn derive_isolated_key(
      master_key: &[u8],
      salt: &[u8],
      purpose: &str,
      length: usize,
  ) -> Result<Vec<u8>, AideError>
  ```

- Dùng HKDF-SHA256 để tách key theo `purpose`, ví dụ:

  - `purpose = "hook:<hook_id>"`
  - `purpose = "payload:<qpid_id>"`

- Đảm bảo các purpose khác nhau sinh ra key khác nhau dù dùng chung `master_key` + `salt`.

### 3. DIM Engine

- Mô hình overlay:

  ```rust
  pub struct DimOverlay {
      pub id: String,
      pub priority: u8,
      pub policy: Option<String>,
      pub time_slot: Option<u64>,
      pub location: Option<String>,
      pub expires_at: Option<u64>,
  }

  pub struct DimContext {
      pub policy: Option<String>,
      pub time_slot: Option<u64>,
      pub location: Option<String>,
  }
  ```

- Hàm:

  ```rust
  pub fn resolve_overlays(overlays: Vec<DimOverlay>, now: u64) -> DimContext
  ```

  - Loại bỏ overlay đã hết hạn (`expires_at <= now`).
  - Sắp xếp theo `priority` (nhỏ hơn = ưu tiên hơn), sau đó theo `id` để deterministic.
  - Gộp policy/time/location từ dưới lên trên (overlay sau có thể override overlay trước).

### 4. FIID Protocol Router + SUL Entry

- Hỗ trợ parse các URI:

  ```text
  qid://qca-rb/0xDA12/1.AF01.1700000000.abcd.deadbeef.sigxyz
  qia://...
  qed://...
  ```

- Enum:

  ```rust
  pub enum FiidProtocol {
      Qid(QidRequest),
      Qia(String),
      Qed(String),
  }

  pub struct SulEntry {
      pub srid_alias: String,
      pub crypto_header: CryptoHeader,
      pub dim: DimContext,
  }
  ```

- API chính:

  ```rust
  pub fn parse_fiid_uri(uri: &str) -> Result<FiidProtocol, AideError>;

  pub fn build_sul_entry(
      uri: &str,
      header_bytes: &[u8],
      overlays: Vec<DimOverlay>,
      now: u64,
  ) -> Result<SulEntry, AideError>;
  ```

- Mục tiêu: từ một QID + CryptoHeader + tập DIM overlay → tạo ra một `SulEntry` cho AIDE runtime.

### 5. SRID Registry (alias → descriptor)

- Registry đơn giản:

  ```rust
  pub struct SridRegistry {
      map: HashMap<String, String>,
  }

  impl SridRegistry {
      pub fn with_defaults() -> SridRegistry {
          // ví dụ:
          // "0xDA12" -> "SHARD_12.DA_PLANE.ROCKBASE.VN.v1"
      }

      pub fn resolve(&self, alias: &str) -> Option<String>;
  }
  ```

- Dùng làm cầu nối giữa FIID (QID/QED) và Rockbase/SDM.

---

## Cài đặt & chạy test

Yêu cầu:

- Rust 1.80+ (toolchain stable)
- Cargo

Clone repo và chạy test:

```bash
git clone https://github.com/hugaalfatech/aide-core.git
cd aide-core
cargo test
```

---

## Ví dụ nhanh

```rust
use aide_core::{
    CryptoHeader,
    DimOverlay,
    build_sul_entry,
    derive_isolated_key,
};

fn demo() {
    let uri = "qid://qca-rb/0xDA12/1.AF01.1700000000.abcd.deadbeef.sigxyz";

    let header = CryptoHeader {
        version: 1,
        hook_alg: 1,
        payload_alg: 1,
        ze_type: 1,
        ze_version: 1,
        flags: 0,
    };
    let header_bytes = header.encode();

    let overlays = vec![
        DimOverlay {
            id: "policy-base".to_string(),
            priority: 5,
            policy: Some("allow".to_string()),
            time_slot: Some(10),
            location: Some("HCM".to_string()),
            expires_at: None,
        },
    ];

    let sul = build_sul_entry(uri, &header_bytes, overlays, 0).unwrap();

    let master = b"master-key";
    let salt = b"ale-or-aop-salt";
    let hook_key = derive_isolated_key(master, salt, "hook:main", 32).unwrap();
    let payload_key = derive_isolated_key(master, salt, "payload:qpid", 32).unwrap();

    assert_ne!(hook_key, payload_key);
    println!("SUL alias: {}", sul.srid_alias);
}
```

---

## Trạng thái dự án

- **Status**: Experimental / Prototype
- Mục tiêu: làm nền cho các dự án Era 5 khác như **SRME**, **SDM**, **Rockbase**, **AIDE runtime**.

Đừng dùng trực tiếp cho production nếu chưa review thêm về security & protocol compatibility.