const { invoke } = window.__TAURI__.tauri;

let logAdmindStdoutEl;
let logAdmindStderrEl;
let logNetlinkdStdoutEl;
let logNetlinkdStderrEl;
let logNetfilterdStdoutEl;
let logNetfilterdStderrEl;
let logDhcp4dStdoutEl;
let logDhcp4dStderrEl;
let logPppoe3StdoutEl;
let logPppoe3StderrEl;
let logDhcp6StdoutEl;
let logDhcp6StderrEl;
let logDnsdStdoutEl;
let logDnsdStderrEl;
let logDsliteStdoutEl;
let logDsliteStderrEl;
let logNtpStdoutEl;
let logNtpStderrEl;
let logRadvdStdoutEl;
let logRadvdStderrEl;
let logNetdumpdStdoutEl;
let logNetdumpdStderrEl;
let logDyndnsStdoutEl;
let logDyndnsStderrEl;
let logWgdStdoutEl;
let logWgdStderrEl;

async function logRead(logfile) {
  return await invoke("log_read", { logfile: logfile });
}

async function loadNonDns() {
  logAdmindStdoutEl.value = await logRead("rustkrazy_admind.log");
  logAdmindStderrEl.value = await logRead("rustkrazy_admind.err");
  logNetlinkdStdoutEl.value = await logRead("rsdsl_netlinkd.log");
  logNetlinkdStderrEl.value = await logRead("rsdsl_netlinkd.err");
  logNetfilterdStdoutEl.value = await logRead("rsdsl_netfilterd.log");
  logNetfilterdStderrEl.value = await logRead("rsdsl_netfilterd.err");
  logDhcp4dStdoutEl.value = await logRead("rsdsl_dhcp4d.log");
  logDhcp4dStderrEl.value = await logRead("rsdsl_dhcp4d.err");
  logPppoe3StdoutEl.value = await logRead("rsdsl_pppoe3.log");
  logPppoe3StderrEl.value = await logRead("rsdsl_pppoe3.err");
  logDhcp6StdoutEl.value = await logRead("rsdsl_dhcp6.log");
  logDhcp6StderrEl.value = await logRead("rsdsl_dhcp6.err");
  logDsliteStdoutEl.value = await logRead("rsdsl_dslite.log");
  logDsliteStderrEl.value = await logRead("rsdsl_dslite.err");
  logNtpStdoutEl.value = await logRead("rsdsl_ntp.log");
  logNtpStderrEl.value = await logRead("rsdsl_ntp.err");
  logRadvdStdoutEl.value = await logRead("rsdsl_radvd.log");
  logRadvdStderrEl.value = await logRead("rsdsl_radvd.err");
  logNetdumpdStdoutEl.value = await logRead("rsdsl_netdumpd.log");
  logNetdumpdStderrEl.value = await logRead("rsdsl_netdumpd.err");
  logDyndnsStdoutEl.value = await logRead("dyndns.log");
  logDyndnsStderrEl.value = await logRead("dyndns.err");
  logWgdStdoutEl.value = await logRead("rsdsl_wgd.log");
  logWgdStderrEl.value = await logRead("rsdsl_wgd.err");

  logAdmindStdoutEl.scrollTop = logAdmindStdoutEl.scrollHeight;
  logAdmindStderrEl.scrollTop = logAdmindStderrEl.scrollHeight;
  logNetlinkdStdoutEl.scrollTop = logNetlinkdStdoutEl.scrollHeight;
  logNetlinkdStderrEl.scrollTop = logNetlinkdStderrEl.scrollHeight;
  logNetfilterdStdoutEl.scrollTop = logNetfilterdStdoutEl.scrollHeight;
  logNetfilterdStderrEl.scrollTop = logNetfilterdStderrEl.scrollHeight;
  logDhcp4dStdoutEl.scrollTop = logDhcp4dStdoutEl.scrollHeight;
  logDhcp4dStderrEl.scrollTop = logDhcp4dStderrEl.scrollHeight;
  logPppoe3StdoutEl.scrollTop = logPppoe3StdoutEl.scrollHeight;
  logPppoe3StderrEl.scrollTop = logPppoe3StderrEl.scrollHeight;
  logDhcp6StdoutEl.scrollTop = logDhcp6StdoutEl.scrollHeight;
  logDhcp6StderrEl.scrollTop = logDhcp6StderrEl.scrollHeight;
  logDsliteStdoutEl.scrollTop = logDsliteStdoutEl.scrollHeight;
  logDsliteStderrEl.scrollTop = logDsliteStderrEl.scrollHeight;
  logNtpStdoutEl.scrollTop = logNtpStdoutEl.scrollHeight;
  logNtpStderrEl.scrollTop = logNtpStderrEl.scrollHeight;
  logRadvdStdoutEl.scrollTop = logRadvdStdoutEl.scrollHeight;
  logRadvdStderrEl.scrollTop = logRadvdStderrEl.scrollHeight;
  logNetdumpdStdoutEl.scrollTop = logNetdumpdStdoutEl.scrollHeight;
  logNetdumpdStderrEl.scrollTop = logNetdumpdStderrEl.scrollHeight;
  logDyndnsStdoutEl.scrollTop = logDyndnsStdoutEl.scrollHeight;
  logDyndnsStderrEl.scrollTop = logDyndnsStderrEl.scrollHeight;
  logWgdStdoutEl.scrollTop = logWgdStdoutEl.scrollHeight;
  logWgdStderrEl.scrollTop = logWgdStderrEl.scrollHeight;
}

window.addEventListener("DOMContentLoaded", async function() {
  logAdmindStdoutEl = document.querySelector("#log-admind-stdout");
  logAdmindStderrEl = document.querySelector("#log-admind-stderr");
  logNetlinkdStdoutEl = document.querySelector("#log-netlinkd-stdout");
  logNetlinkdStderrEl = document.querySelector("#log-netlinkd-stderr");
  logNetfilterdStdoutEl = document.querySelector("#log-netfilterd-stdout");
  logNetfilterdStderrEl = document.querySelector("#log-netfilterd-stderr");
  logDhcp4dStdoutEl = document.querySelector("#log-dhcp4d-stdout");
  logDhcp4dStderrEl = document.querySelector("#log-dhcp4d-stderr");
  logPppoe3StdoutEl = document.querySelector("#log-pppoe3-stdout");
  logPppoe3StderrEl = document.querySelector("#log-pppoe3-stderr");
  logDhcp6StdoutEl = document.querySelector("#log-dhcp6-stdout");
  logDhcp6StderrEl = document.querySelector("#log-dhcp6-stderr");
  logDnsdStdoutEl = document.querySelector("#log-dnsd-stdout");
  logDnsdStderrEl = document.querySelector("#log-dnsd-stderr");
  logDsliteStdoutEl = document.querySelector("#log-dslite-stdout");
  logDsliteStderrEl = document.querySelector("#log-dslite-stderr");
  logNtpStdoutEl = document.querySelector("#log-ntp-stdout");
  logNtpStderrEl = document.querySelector("#log-ntp-stderr");
  logRadvdStdoutEl = document.querySelector("#log-radvd-stdout");
  logRadvdStderrEl = document.querySelector("#log-radvd-stderr");
  logNetdumpdStdoutEl = document.querySelector("#log-netdumpd-stdout");
  logNetdumpdStderrEl = document.querySelector("#log-netdumpd-stderr");
  logDyndnsStdoutEl = document.querySelector("#log-dyndns-stdout");
  logDyndnsStderrEl = document.querySelector("#log-dyndns-stderr");
  logWgdStdoutEl = document.querySelector("#log-wgd-stdout");
  logWgdStderrEl = document.querySelector("#log-wgd-stderr");

  loadNonDns();
  // logDnsdStdoutEl.value = await logRead("rsdsl_dnsd.log");
  // logDnsdStderrEl.value = await logRead("rsdsl_dnsd.err");

  // document.querySelector("#log-dnsd-refresh").addEventListener("click", async function(e) {
  // 	e.preventDefault();
  //   logDnsdStdoutEl.value = await logRead("rsdsl_dnsd.log");
  //   logDnsdStderrEl.value = await logRead("rsdsl_dnsd.err");
  // });
});

setInterval(loadNonDns, 5000);
