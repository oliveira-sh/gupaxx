// Gupaxx - Fork of Gupax
//
// Copyright (c) 2024-2025 Cyrix126
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::sync::{Arc, Mutex};

use egui::{Align, Image, Label, RichText, ScrollArea, TextStyle, Ui};
use log::debug;
use readable::num::Float;
use readable::up::Uptime;
use strum::EnumCount;

use crate::app::panels::middle::common::console::console;
use crate::app::panels::middle::common::header_tab::header_tab;
use crate::app::panels::middle::common::state_edit_field::StateTextEdit;
use crate::app::panels::middle::common::toggle::toggle_ui_compact;
use crate::disk::state::{ManualDonationLevel, ManualDonationMetric, XvbMode};
use crate::helper::ProcessName;
use crate::helper::xrig::xmrig::PubXmrigApi;
use crate::helper::xrig::xmrig_proxy::PubXmrigProxyApi;
use crate::helper::xvb::PubXvbApi;
use crate::helper::xvb::priv_stats::RuntimeMode;
use crate::miscs::height_txt_before_button;
use crate::utils::constants::{
    ORANGE, XVB_DONATED_1H_FIELD, XVB_DONATED_24H_FIELD, XVB_DONATION_LEVEL_DONOR_HELP,
    XVB_DONATION_LEVEL_MEGA_DONOR_HELP, XVB_DONATION_LEVEL_VIP_DONOR_HELP,
    XVB_DONATION_LEVEL_WHALE_DONOR_HELP, XVB_FAILURE_FIELD, XVB_HELP, XVB_HERO_SELECT,
    XVB_MANUAL_POOL, XVB_MANUAL_SLIDER_MANUAL_P2POOL_HELP, XVB_MANUAL_SLIDER_MANUAL_XVB_HELP,
    XVB_MODE_MANUAL_DONATION_LEVEL_HELP, XVB_MODE_MANUAL_P2POOL_HELP, XVB_MODE_MANUAL_XVB_HELP,
    XVB_ROUND_TYPE_FIELD, XVB_TOKEN_LEN, XVB_URL_RULES, XVB_WINNER_FIELD,
};
use crate::utils::regex::Regexes;
use crate::{XVB_MINING_ON_FIELD, XVB_P2POOL_BUFFER, XVB_SIDECHAIN};
use crate::{
    constants::{BYTES_XVB, SPACE},
    utils::constants::XVB_URL,
};

impl crate::disk::state::Xvb {
    #[inline(always)] // called once
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        &mut self,
        address: &str,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
        api: &Arc<Mutex<PubXvbApi>>,
        gui_api_xmrig: &Arc<Mutex<PubXmrigApi>>,
        gui_api_xp: &Arc<Mutex<PubXmrigProxyApi>>,
        is_alive: bool,
    ) {
        // logo and website link
        let logo = Some(Image::from_bytes("bytes:/xvb.png", BYTES_XVB));
        header_tab(
            ui,
            logo,
            &[
                ("XMRvsBEAST", XVB_URL, ""),
                (
                    "Rules",
                    XVB_URL_RULES,
                    "Click here to read the rules and understand how the raffle works.",
                ),
                ("FAQ", "https://xmrvsbeast.com/p2pool/faq.html", ""),
            ],
            None,
            true,
        );
        egui::ScrollArea::vertical().show(ui, |ui| {

            // console output for log
            debug!("XvB Tab | Rendering [Console]");
            ui.group(|ui| {
                let text = &api.lock().unwrap().output;
                console(ui, text, &mut self.console_height, ProcessName::Xvb);
            });
            // input token
            ui.add_space(SPACE);
            ui.horizontal(|ui| {
                ui.group(|ui|{
                    ui.style_mut().override_text_valign = Some(Align::Center);
                    self.field_token(ui);
                });

        // --------------------------- XVB Simple -------------------------------------------
        if self.simple {
            ui.add_space(SPACE);
            ui.checkbox(&mut self.simple_hero_mode, "Hero Mode").on_hover_text(XVB_HERO_SELECT);
            // set runtime mode immediately if we are on simple mode.
            if self.simple_hero_mode {
                api.lock().unwrap().stats_priv.runtime_mode = RuntimeMode::Hero;
            }  else {
                api.lock().unwrap().stats_priv.runtime_mode = RuntimeMode::Auto;
            }
        }
    });
        ui.add_space(SPACE);
         // --------------------------- XVB Advanced -----------------------------------------
                        let text_height = height_txt_before_button(ui, &TextStyle::Heading) * 1.4;
                ScrollArea::horizontal().id_salt("horizontal").show(ui, |ui| {
         if !self.simple {

            ui.group(|ui| {
                ui.set_width(0.0);
                ui.vertical(|ui| {
                        ui.style_mut().override_text_valign = Some(Align::Center);
                        ui.set_height(0.0);
                        ui.set_height(0.0);
                        egui::ComboBox::from_label("").height(XvbMode::COUNT as f32 * (ui.text_style_height(&TextStyle::Button) + (ui.spacing().button_padding.y * 2.0) + ui.spacing().item_spacing.y))
                        .selected_text(self.mode.to_string())
                        .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.mode, XvbMode::Auto,
                                     XvbMode::Auto.to_string());
                                ui.selectable_value(&mut self.mode, XvbMode::Hero,
                                     XvbMode::Hero.to_string()).on_hover_text(XVB_HERO_SELECT);
                                ui.selectable_value(&mut self.mode, XvbMode::ManualXvb,
                                     XvbMode::ManualXvb.to_string())
                                .on_hover_text(XVB_MODE_MANUAL_XVB_HELP);
                                ui.selectable_value(&mut self.mode, XvbMode::ManualP2pool,
                                     XvbMode::ManualP2pool.to_string())
                                .on_hover_text(XVB_MODE_MANUAL_P2POOL_HELP);
                                ui.selectable_value(&mut self.mode, XvbMode::ManualDonationLevel,
                                     XvbMode::ManualDonationLevel.to_string())
                                .on_hover_text(XVB_MODE_MANUAL_DONATION_LEVEL_HELP);
                        });
                        if self.mode == XvbMode::ManualXvb || self.mode == XvbMode::ManualP2pool {

                            ui.add_space(SPACE);

                            let default_xmrig_hashrate = match self.manual_donation_metric {
                                ManualDonationMetric::Hash => 1_000.0,
                                ManualDonationMetric::Kilo => 1_000_000.0,
                                ManualDonationMetric::Mega => 1_000_000_000.0
                            };
                            // use proxy HR in priority, or use xmrig or default.
                            let mut hashrate_xmrig = {
                                if gui_api_xp.lock().unwrap().hashrate_10m > 0.0 {
                                    gui_api_xp.lock().unwrap().hashrate_10m
                                } else if gui_api_xmrig.lock().unwrap().hashrate_raw_15m > 0.0 {
                                    gui_api_xmrig.lock().unwrap().hashrate_raw_15m
                                } else if gui_api_xmrig.lock().unwrap().hashrate_raw_1m > 0.0 {
                                    gui_api_xmrig.lock().unwrap().hashrate_raw_1m
                                } else if gui_api_xmrig.lock().unwrap().hashrate_raw > 0.0 {
                                    gui_api_xmrig.lock().unwrap().hashrate_raw
                                } else {
                                    default_xmrig_hashrate
                                }
                            };
                            // Adjust maximum slider amount based on slider metric
                            if self.manual_donation_metric == ManualDonationMetric::Kilo {
                                hashrate_xmrig /= 1000.0;
                            } else if self.manual_donation_metric == ManualDonationMetric::Mega {
                                hashrate_xmrig /= 1_000_000.0;
                            }


                            let slider_help_text = if self.mode == XvbMode::ManualXvb {
                                XVB_MANUAL_SLIDER_MANUAL_XVB_HELP
                            } else {
                                XVB_MANUAL_SLIDER_MANUAL_P2POOL_HELP
                            };

                            ui.horizontal(|ui| {

                                if ui.add_sized([0.0, text_height],egui::SelectableLabel::new(self.manual_donation_metric == ManualDonationMetric::Hash, "Hash")).clicked() {
                                    self.manual_donation_metric = ManualDonationMetric::Hash;
                                    self.manual_slider_amount = self.manual_amount_raw;
                                }
                                if ui.add_sized([0.0, text_height],egui::SelectableLabel::new(self.manual_donation_metric == ManualDonationMetric::Kilo, "Kilo")).clicked() {
                                    self.manual_donation_metric = ManualDonationMetric::Kilo;
                                    self.manual_slider_amount = self.manual_amount_raw / 1000.0;
                                };
                                if ui.add_sized([0.0, text_height],egui::SelectableLabel::new(self.manual_donation_metric == ManualDonationMetric::Mega, "Mega")).clicked() {
                                    self.manual_donation_metric = ManualDonationMetric::Mega;
                                    self.manual_slider_amount = self.manual_amount_raw / 1_000_000.0;
                                };
                                ui.spacing_mut().slider_width = ui.text_style_height(&TextStyle::Button) * 18.0;
                                ui.add_sized(
                                    [ui.available_width(), text_height],
                                    egui::Slider::new(&mut self.manual_slider_amount, 0.0..=(hashrate_xmrig as f64))
                                    .text(self.manual_donation_metric.to_string())
                                    .max_decimals(3)
                                ).on_hover_text(slider_help_text);

                            });
            ui.add_space(SPACE);
                        }

                        if self.mode ==  XvbMode::ManualDonationLevel {
                            ui.add_space(SPACE);
                            ui.horizontal(|ui| {
                            ui.radio_value(&mut self.manual_donation_level, ManualDonationLevel::Donor,
                                ManualDonationLevel::Donor.to_string())
                            .on_hover_text(XVB_DONATION_LEVEL_DONOR_HELP);
                            ui.radio_value(&mut self.manual_donation_level, ManualDonationLevel::DonorVIP,
                                ManualDonationLevel::DonorVIP.to_string())
                            .on_hover_text(XVB_DONATION_LEVEL_VIP_DONOR_HELP);
                            ui.radio_value(&mut self.manual_donation_level, ManualDonationLevel::DonorWhale,
                                ManualDonationLevel::DonorWhale.to_string())
                            .on_hover_text(XVB_DONATION_LEVEL_WHALE_DONOR_HELP);
                            ui.radio_value(&mut self.manual_donation_level, ManualDonationLevel::DonorMega,
                                ManualDonationLevel::DonorMega.to_string())
                            .on_hover_text(XVB_DONATION_LEVEL_MEGA_DONOR_HELP);

                            api.lock().unwrap().stats_priv.runtime_manual_donation_level = self.manual_donation_level.clone().into();
                            });
            ui.add_space(SPACE);
                        }
                    });
                });

            // Update manual_amount_raw based on slider
            match self.manual_donation_metric {
                ManualDonationMetric::Hash => {
                    self.manual_amount_raw = self.manual_slider_amount;
                },
                ManualDonationMetric::Kilo => {
                    self.manual_amount_raw = self.manual_slider_amount * 1000.0;
                },
                ManualDonationMetric::Mega => {
                    self.manual_amount_raw = self.manual_slider_amount * 1_000_000.0;
                }
            }

            // Set runtime_mode & runtime_manual_amount
            api.lock().unwrap().stats_priv.runtime_mode = self.mode.clone().into();
            api.lock().unwrap().stats_priv.runtime_manual_amount = self.manual_amount_raw;
         ui.add_space(SPACE);
        let p2pool_buffer_enabled = matches!(self.mode, XvbMode::Auto | XvbMode::Hero);


         ui.horizontal(|ui|{
            // allow user to modify the buffer for p2pool
            // button
            ui.add_enabled_ui(p2pool_buffer_enabled, |ui|{
 ui.add_sized(
                [0.0 , text_height],
                egui::Slider::new(&mut self.p2pool_buffer, -100..=100)
                .text("% P2Pool Buffer" )
            ).on_hover_text(XVB_P2POOL_BUFFER);

            }).response.on_disabled_hover_text(XVB_P2POOL_BUFFER);

         ui.add_space(SPACE);
         // p2pool sidechain HR or stratum data
            if ui.add_sized(
                [0.0, text_height],
                egui::Checkbox::new(&mut self.use_p2pool_sidechain_hr, "Watch P2Pool Sidechain HR")).on_hover_text(XVB_SIDECHAIN).clicked() {
                api.lock().unwrap().use_p2pool_sidechain_hr = self.use_p2pool_sidechain_hr;
            }
         });
        // Allow user to choose XvB pool manually
        // checkbox to enable
        ui.checkbox(&mut self.manual_pool_enabled, "Manual selection of the XvB pool").on_hover_text(XVB_MANUAL_POOL);
        // slider for EU or NA
            ui.horizontal(|ui|{
        ui.add_enabled_ui(self.manual_pool_enabled, |ui|{
                ui.style_mut().override_text_style = Some(TextStyle::Heading);
                ui.add_sized([0.0, text_height], Label::new(" [ NA"));
                toggle_ui_compact(&mut self.manual_pool_eu, ui);
                ui.add_sized([0.0, text_height], Label::new("EU ]"));
            });
        });

        }

        // need to warn the user if no address is set in p2pool tab
        if !Regexes::addr_ok(address) {
            debug!("XvB Tab | Rendering warning text");
                ui.horizontal_wrapped(|ui|{
            ui.label(RichText::new("You don't have any payout address set in the P2pool Tab ! XvB process needs one to function properly.")
                    .color(ORANGE));
                });
        }
            // private stats
            ui.add_space(SPACE);
            ui.add_enabled_ui(is_alive, |ui| {
                let api = &api.lock().unwrap();
                let priv_stats = &api.stats_priv;
                let current_node = &api.current_pool;
                let style_height = ui.text_style_height(&TextStyle::Body);

        let width_column = ui.text_style_height(&TextStyle::Body) * 12.0;
        let height_column = 0.0;
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            ui.horizontal(|ui| {
                    // Failures
                    stat_box(ui, XVB_FAILURE_FIELD, &priv_stats.fails.to_string(), height_column);
                    stat_box(ui, XVB_DONATED_1H_FIELD,
                                        &[
                                            Float::from_3(priv_stats.donor_1hr_avg as f64).to_string(),
                                            " kH/s".to_string(),
                                        ]
                                        .concat()
                        ,  height_column);
                    stat_box(ui, XVB_DONATED_24H_FIELD,
                                        &[
                                            Float::from_3(priv_stats.donor_24hr_avg as f64).to_string(),
                                            " kH/s".to_string(),
                                        ]
                                        .concat()
                        ,  height_column);
                            ui.add_enabled_ui(priv_stats.round_participate.is_some(), |ui| {
                                 let round = match &priv_stats.round_participate {
                        Some(r) => r.to_string(),
                        None => "None".to_string(),
                    };
                    stat_box(ui, XVB_ROUND_TYPE_FIELD, &round, height_column);
                    }).response
                                .on_disabled_hover_text(
                                    "You do not yet have a share in the PPLNS Window.",
                                );
                    stat_box(ui, XVB_WINNER_FIELD,
if priv_stats.win_current {
                                        "You are Winning the round !"
                                    } else {
                                        "You are not the winner"
                                    }
                        , height_column);
                });
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.set_width(width_column);
                            ui.set_height(height_column);
                            ui.vertical_centered(|ui| {
                                ui.spacing_mut().item_spacing = [style_height / 2.0, style_height / 2.0].into();
                                ui.add_space(SPACE);
                                    ui.label(XVB_MINING_ON_FIELD)
                                        .on_hover_text_at_pointer(&priv_stats.msg_indicator);
                                    ui.label(
                                        current_node
                                            .as_ref()
                                            .map_or("No where".to_string(), |n| n.to_string()),
                                    )
                                    .on_hover_text_at_pointer(&priv_stats.msg_indicator);
                                    ui.label(Uptime::from(priv_stats.time_switch_pool).to_string())
                                        .on_hover_text_at_pointer(&priv_stats.msg_indicator)
                                })
                            });
                    })
                        .response
                        .on_disabled_hover_text("Algorithm is not running.");
                // indicators
                    })
                });
                    // currently mining on
                });
    }
    fn field_token(&mut self, ui: &mut Ui) {
        StateTextEdit::new(ui)
            .help_msg(XVB_HELP)
            .max_ch(XVB_TOKEN_LEN as u8)
            .text_edit_width_same_as_max_ch(ui)
            .description(" Token ")
            .validations(&[|x| x.parse::<u32>().is_ok() && x.len() == XVB_TOKEN_LEN])
            .build(ui, &mut self.token);
    }
}
fn stat_box(ui: &mut Ui, title: &str, value: &str, column_height: f32) {
    ui.vertical(|ui| {
        ui.group(|ui| {
            let width_txt = (title.len().max(value.len()) as f32
                * ui.text_style_height(&TextStyle::Button)
                / 2.0)
                + ui.spacing().item_spacing.x * 2.0;
            ui.set_width(width_txt);
            ui.set_height(column_height);
            ui.vertical_centered(|ui| {
                ui.add_space(SPACE * 3.0);
                ui.label(title);
                ui.label(value);
                ui.add_space(SPACE);
            });
        });
    });
}
