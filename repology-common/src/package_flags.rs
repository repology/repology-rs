// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use bitflags::bitflags;

bitflags! {
    #[derive(Default, Debug, PartialEq, Clone, Copy, Eq)] // XXX: Serialize
    pub struct PackageFlags: u32 {
        const Remove     = 1 << 0;
        const Devel      = 1 << 1;
        const Ignore     = 1 << 2;
        const Incorrect  = 1 << 3;
        const Untrusted  = 1 << 4;
        const NoScheme   = 1 << 5;
        const Rolling    = 1 << 7;
        const Sink       = 1 << 8;
        const Legacy     = 1 << 9;
        const PIsPatch   = 1 << 10;
        const AnyIsPatch = 1 << 11;
        const Trace      = 1 << 12;
        const WeakDevel  = 1 << 13;
        const Stable     = 1 << 14;
        const AltVer     = 1 << 15;
        const Vulnerable = 1 << 16;
        const AltScheme  = 1 << 17;
        const NoLegacy   = 1 << 18;
        const Outdated   = 1 << 19;
        const Recalled   = 1 << 20;
    }
}
