/*
GNU General Public License v3.0+ (see LICENSES/GPL-3.0-or-later.txt or https://www.gnu.org/licenses/gpl-3.0.txt)
SPDX-FileCopyrightText: 2024, Felix Fontein
SPDX-License-Identifier: GPL-3.0-or-later
*/

use antsibull::util::IntoString;

fn main() {
    let context = &antsibull::markup::Context {
        current_plugin: Option::None,
        role_entrypoint: Option::None,
    };
    let f = antsibull::markup::parse(
        "The B(module) that I(is) C(defined) in M(ansible.builtin.debug) is L(called, https://docs.ansible.com/ansible/latest/collections/ansible/builtin/debug_module.html) U(https://docs.ansible.com/ansible/latest/), O(foo[].bar[3].baz=bam).",
        // "The B(module) that I(is) C(defined) V(fo\\o)o)",
        &context,
        &antsibull::markup::ParseOptions::default(),
    );
    println!("Antsibull says hello from Rust: {:?}", f);
    println!("Nicer:");
    for x in &f {
        println!("  {}", x);
    }
    let mut appender = antsibull::util::CollectorAppender::new();
    antsibull::markup::append_md_paragraph(
        &mut appender,
        f.iter().map(|ps| &ps.part),
        &antsibull::markup::NoLinkProvider::new(),
        &None,
    );
    println!("Result: {}", appender.into_string());
}
