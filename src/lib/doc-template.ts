import { settingsCommands } from "@/db/commands";
import { formatSequentialNumber } from "@/lib/utils";

const HEADER_URL = "https://res.cloudinary.com/dowbdhnew/image/upload/v1786300669/r_gsluo8.png";
const LOGO_URL = "https://res.cloudinary.com/dowbdhnew/image/upload/v1786103303/abc-removebg-preview_mfrmwi.png";
const FOOTER_URL = "https://res.cloudinary.com/dowbdhnew/image/upload/v1786300588/l_udlsuc.png";

function buildDocument(title: string, rows: { label: string; value: string }[]): string {
  const rowsHtml = rows
    .map(
      (r) => `
      <tr>
        <td class="data-row"><span class="label">${r.label} :</span><span class="value">${r.value}</span></td>
      </tr>`,
    )
    .join("\n");

  return `<!DOCTYPE html>
<html lang="ar" dir="rtl">
<head>
  <meta charset="UTF-8" />
  <title>${title}</title>
  <style>
    @page { size: A4; margin: 0; }
    * { box-sizing: border-box; }
    html, body { margin: 0; padding: 0; background: #fff; }
    body { font-family: "Amiri", "Traditional Arabic", "Times New Roman", serif; direction: rtl; }
    .paper { width: 210mm; min-height: 297mm; margin: 0 auto; padding: 18mm 22mm; background: #fff; }
    .header { width: 100%; display: flex; align-items: flex-start; justify-content: space-between; direction: rtl; }
    .header-column { display: flex; align-items: flex-start; }
    .header-right { width: 40%; justify-content: flex-start; }
    .header-center { width: 20%; justify-content: center; }
    .header-left { width: 40%; justify-content: flex-end; }
    .header-right img { width: 120px; height: auto; display: block; }
    .header-center img { width: 100px; height: auto; display: block; }
    .header-left img { width: 175px; height: auto; display: block; }
    .doc-title { text-align: center; font-size: 24px; font-weight: bold; margin: 80px 0 70px; color: #1a1a1a; }
    .data-table { width: 100%; border-collapse: collapse; margin-top: 20px; }
    .data-table tr { border-bottom: none; }
    .data-table td { padding: 8px 16px; vertical-align: top; }
    .data-table .data-row { direction: rtl; }
    .data-table .label { font-weight: 900; font-size: 22px; color: #1a1a1a; margin-left: 14px; }
    .data-table .value { font-weight: normal; font-size: 22px; color: #374151; }
    @media print {
      html, body { width: 210mm; min-height: 297mm; background: #fff; }
      .paper { width: 210mm; min-height: 297mm; margin: 0; padding: 18mm 22mm; }
    }
  </style>
</head>
<body>
  <div class="paper">
    <header class="header">
      <div class="header-column header-right">
        <img src="${HEADER_URL}" alt="رأس الصفحة" />
      </div>
      <div class="header-column header-center">
        <img src="${LOGO_URL}" alt="شعار المملكة المغربية" />
      </div>
      <div class="header-column header-left">
        <img src="${FOOTER_URL}" alt="رأس الصفحة" />
      </div>
    </header>
    <h1 class="doc-title">${title}</h1>
    <table class="data-table">
      ${rowsHtml}
    </table>
  </div>
</body>
</html>`;
}

function formatEnDate(iso: string): string {
  if (!iso) return "—";
  try {
    const d = new Date(iso);
    return `${d.getFullYear()}/${String(d.getMonth() + 1).padStart(2, "0")}/${String(d.getDate()).padStart(2, "0")}`;
  } catch {
    return iso.slice(0, 10);
  }
}

export function printIncoming(item: { registration_number: string; is_duplicate: boolean; date: string; correspondence_number: string | null; arrival_date: string | null; subject: string; sender: string; destination_service: string; notes: string | null }) {
  const html = buildDocument(`سجل الوارد`, [
    { label: "الرقم الترتيبي", value: formatSequentialNumber(item.registration_number, item.is_duplicate) },
    { label: "رقم الرسالة", value: item.correspondence_number || "—" },
    { label: "تاريخ الرسالة", value: formatEnDate(item.date) },
    { label: "تاريخ الوصول", value: formatEnDate(item.arrival_date ?? "") },
    { label: "اسم و موطن المرسل", value: item.sender },
    { label: "الموضوع", value: item.subject },
  ]);
  settingsCommands.saveAndOpenHtml(`wareed_${item.registration_number}.html`, html);
}

export function printOutgoing(item: { registration_number: string; date: string; recipient: string; subject: string; correspondence_number: string | null; source: string | null; notes: string | null }) {
  const html = buildDocument(`سجل المراسلة`, [
    { label: "التاريخ", value: formatEnDate(item.date) },
    { label: "المرسل إليه", value: item.recipient },
    { label: "الموضوع", value: item.subject },
    { label: "التاريخ الوارد", value: item.correspondence_number ? formatEnDate(item.correspondence_number) : "—" },
    { label: "المصدر والجواب", value: item.source || "—" },
    { label: "النتيجة", value: item.notes || "—" },
  ]);
  settingsCommands.saveAndOpenHtml(`murasala_${item.registration_number}.html`, html);
}
