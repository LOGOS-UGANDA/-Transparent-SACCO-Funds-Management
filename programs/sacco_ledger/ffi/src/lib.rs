use std::ffi::{CStr, CString};
use std::os::raw::c_char;

fn encode_request(value: &str) -> Result<serde_json::Value, String> {
    let instruction: sacco_ledger_core::Instruction =
        serde_json::from_str(value).map_err(|error| error.to_string())?;
    let bytes = borsh::to_vec(&instruction).map_err(|error| error.to_string())?;
    Ok(serde_json::json!({ "ok": true, "data": bytes }))
}

fn account_request(name: &str) -> Result<serde_json::Value, String> {
    let accounts = sacco_ledger_idl::instruction_accounts_json(name)
        .ok_or_else(|| format!("unknown instruction: {name}"))?;
    let data: serde_json::Value =
        serde_json::from_str(&accounts).map_err(|error| error.to_string())?;
    Ok(serde_json::json!({ "ok": true, "data": data }))
}

fn decode_request(value: &str) -> Result<serde_json::Value, String> {
    let request: serde_json::Value =
        serde_json::from_str(value).map_err(|error| error.to_string())?;
    let kind = request
        .get("kind")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "missing account kind".to_owned())?;
    let bytes = request
        .get("bytes")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "missing shard bytes".to_owned())?
        .iter()
        .map(|byte| {
            byte.as_u64()
                .filter(|value| *value <= 255)
                .map(|value| value as u8)
                .ok_or_else(|| "invalid shard byte".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    fn decode<T: borsh::BorshDeserialize + serde::Serialize>(
        bytes: &[u8],
    ) -> Result<serde_json::Value, String> {
        let value = T::try_from_slice(bytes).map_err(|error| error.to_string())?;
        serde_json::to_value(value).map_err(|error| error.to_string())
    }
    let data = match kind {
        "sacco" => decode::<sacco_ledger_core::Sacco>(&bytes)?,
        "member" => decode::<sacco_ledger_core::Member>(&bytes)?,
        "deposit" => decode::<sacco_ledger_core::Deposit>(&bytes)?,
        "loan" => decode::<sacco_ledger_core::Loan>(&bytes)?,
        "expense" => decode::<sacco_ledger_core::Expense>(&bytes)?,
        "summary" => decode::<sacco_ledger_core::PeriodSummary>(&bytes)?,
        "config" => decode::<sacco_ledger_core::ProgramConfig>(&bytes)?,
        _ => return Err("unknown account kind".to_owned()),
    };
    Ok(serde_json::json!({ "ok": true, "data": data }))
}

fn config_pda_request(value: &str) -> Result<serde_json::Value, String> {
    use std::str::FromStr;
    let program_id =
        sacco_ledger_core::AccountId::from_str(value).map_err(|error| error.to_string())?;
    let config_id = sacco_ledger_core::compute_config_pda(program_id).to_string();
    Ok(serde_json::json!({ "ok": true, "data": config_id }))
}

#[allow(unsafe_code)]
mod ffi {
    use super::*;
    use std::ptr;

    fn respond(value: Result<serde_json::Value, String>) -> *mut c_char {
        let response =
            value.unwrap_or_else(|error| serde_json::json!({ "ok": false, "error": error }));
        match CString::new(response.to_string()) {
            Ok(response) => response.into_raw(),
            Err(_) => ptr::null_mut(),
        }
    }

    unsafe fn input(value: *const c_char) -> Result<String, String> {
        if value.is_null() {
            return Err("null input".to_owned());
        }
        // SAFETY: callers pass a valid NUL-terminated UTF-8 buffer for the call duration.
        unsafe { CStr::from_ptr(value) }
            .to_str()
            .map(str::to_owned)
            .map_err(|error| error.to_string())
    }

    #[no_mangle]
    pub unsafe extern "C" fn sacco_ledger_encode_instruction_json(
        value: *const c_char,
    ) -> *mut c_char {
        respond(unsafe { input(value) }.and_then(|value| encode_request(&value)))
    }

    #[no_mangle]
    pub unsafe extern "C" fn sacco_ledger_instruction_accounts_json(
        value: *const c_char,
    ) -> *mut c_char {
        respond(unsafe { input(value) }.and_then(|value| account_request(&value)))
    }

    #[no_mangle]
    pub unsafe extern "C" fn sacco_ledger_decode_shard_json(value: *const c_char) -> *mut c_char {
        respond(unsafe { input(value) }.and_then(|value| decode_request(&value)))
    }

    #[no_mangle]
    pub unsafe extern "C" fn sacco_ledger_config_pda_json(value: *const c_char) -> *mut c_char {
        respond(unsafe { input(value) }.and_then(|value| config_pda_request(&value)))
    }

    #[no_mangle]
    pub unsafe extern "C" fn sacco_ledger_client_free(value: *mut c_char) {
        if !value.is_null() {
            // SAFETY: `value` must be a pointer returned by one of the functions above.
            drop(unsafe { CString::from_raw(value) });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{account_request, encode_request};

    #[test]
    fn encodes_the_rust_instruction_enum_as_borsh() {
        let response = encode_request(r#"{"Initialize":null}"#).expect("valid instruction");
        assert_eq!(response["data"], serde_json::json!([0]));
    }

    #[test]
    fn exposes_account_order_and_signers_from_the_idl_table() {
        let response = account_request("record_deposit").expect("IDL entry exists");
        assert_eq!(response["data"][0]["name"], "sacco");
        assert_eq!(response["data"][2]["name"], "deposit");
        assert_eq!(response["data"][2]["signer"], true);
    }
}
