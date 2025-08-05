use std::ptr::NonNull;

use super::ChrIns;
use shared::OwnedPtr;

#[repr(C)]
/// Manages speffects for an entity.
///
/// Source of name: RTTI
pub struct SpecialEffect {
    vftable: usize,
    head: Option<OwnedPtr<SpecialEffectEntry>>,
    /// ChrIns this SpecialEffect structure belongs to.
    pub owner: NonNull<ChrIns>,
    unk18: usize,
    unk20: [u8; 0x118],
}

type AddSpeffectFunction = unsafe extern "C" fn(player_ins: *mut u8, speffect_id: u32, param3: i32);

static ADD_SPEFFECT_FUNCTION: once_cell::sync::Lazy<Option<AddSpeffectFunction>> =
    once_cell::sync::Lazy::new(|| unsafe {
        use skidscan::signature;
        signature!(
        "48 8B C4 48 89 58 08 48 89 70 10 57 48 81 EC ?? ?? ?? ?? 0F 28 05 ?? ?? ?? ?? 48 8B F1 0F 28 0D ?? ?? ?? ?? 48 8D 48 88"
    )
        .scan_module("eldenring.exe")
        .ok()
        .map(|ptr| unsafe { std::mem::transmute(ptr) })
    });

impl SpecialEffect {
    /// Yields an iterator over all the SpEffect entries contained in this SpecialEffect instance.
    pub fn entries(&self) -> impl Iterator<Item = &SpecialEffectEntry> {
        let mut current = self.head.as_ref().map(|e| e.as_ptr());

        std::iter::from_fn(move || {
            let ret = current.and_then(|c| unsafe { c.as_ref() });
            current = unsafe { ret?.next.map(|e| e.as_ptr()) };
            ret
        })
    }

    pub unsafe fn add_speffect(&self, sp_effect_id: u32, player_index: u8) {
        let player_ptr = self.owner.as_ptr() as *mut u8;

        if player_ptr.is_null() {
            eprintln!("Player pointer is null");
            return;
        }

        let Some(func) = *ADD_SPEFFECT_FUNCTION else {
            eprintln!("AddSpeffectFunction is not initialized");
            return;
        };

        func(player_ptr, sp_effect_id, player_index as i32);
    }
}

#[repr(C)]
/// Represents an active SpEffect.
pub struct SpecialEffectEntry {
    /// The param row this speffect entry uses.
    param_data: usize,
    /// The param ID for this speffect entry.
    pub param_id: u32,
    _padc: u32,
    pub accumulator_info: SpecialEffectEntryAccumulatorInfo,
    /// The next param entry in the doubly linked list.
    next: Option<NonNull<SpecialEffectEntry>>,
    /// The previous param entry in the doubly linked list.
    previous: Option<NonNull<SpecialEffectEntry>>,
    /// Time to go until the speffect is removed.
    unk_removal_timer: f32,
    pub removal_timer: f32,
    /// How long it takes the speffect before removing itself.
    pub duration: f32,
    pub interval_timer: f32,
    unk50: [u8; 0x28],
}

#[repr(C)]
/// Source of name: RTTI
pub struct SpecialEffectEntryAccumulatorInfo {
    unk0: usize,
    pub upper_trigger_count: i32,
    pub effect_on_upper_or_higher: i32,
    pub lower_trigger_count: i32,
    pub effect_on_lower_or_below: i32,
    unk18: i32,
    unk1c: u32,
}

#[repr(C)]
pub struct NpcSpEffectEquipCtrl {
    pub sp_effect_equip_ctrl: SpEffectEquipCtrl,
}

#[repr(C)]
pub struct SpEffectEquipCtrl {
    vfptr: usize,
    /// Whatever ChrIns this equip ctrl is tied to.
    pub owner: NonNull<ChrIns>,
    /// The owning ChrIns's SpEffect.
    pub sp_effect: NonNull<SpecialEffect>,
}
