import { useState } from "react";
import { X, Loader2, FileSpreadsheet, CheckCircle2, AlertTriangle } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { useQueryClient } from "@tanstack/react-query";
import { excelCommands } from "@/db/commands";
import type {
  ExcelAnalysis,
  ExcelColumn,
  ExcelImportResult,
} from "@/db/commands";

interface Props {
  onClose: () => void;
}

const FIELD_LABELS: { field: string; label: string }[] = [
  { field: "registration_number", label: "الرقم الترتيبي" },
  { field: "correspondence_number", label: "رقمها" },
  { field: "date", label: "تاريخ الرسالة" },
  { field: "arrival_date", label: "تاريخ الوصول" },
  { field: "subject", label: "الموضوع" },
  { field: "sender", label: "اسم و موطن المرسل" },
  { field: "recipient", label: "المستلم" },
  { field: "destination_service", label: "المصلحة" },
  { field: "source", label: "المصدر والجواب" },
  { field: "notes", label: "النتيجة / ملاحظات" },
];

function fieldName(field: string | null): string {
  if (!field) return "";
  const found = FIELD_LABELS.find((f) => f.field === field);
  return found ? found.label : field;
}

const MAX_PREVIEW = 8;

export function ExcelImport({ onClose }: Props) {
  const qc = useQueryClient();

  const [analyzing, setAnalyzing] = useState(false);
  const [importing, setImporting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [analysis, setAnalysis] = useState<ExcelAnalysis | null>(null);
  const [kind, setKind] = useState<string>("incoming");
  const [restored, setRestored] = useState<string[]>([]);
  const [result, setResult] = useState<ExcelImportResult | null>(null);

  const handlePick = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          { name: "Excel", extensions: ["xlsx", "xls"] },
          { name: "كل الملفات", extensions: ["*"] },
        ],
      });
      if (!selected || typeof selected !== "string") return;

      setAnalyzing(true);
      setError(null);
      setAnalysis(null);
      setResult(null);

      const a = await excelCommands.analyze(selected);
      setAnalysis(a);
      setKind(a.kind);
      setRestored(a.columns.map((c) => c.field ?? ""));
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setAnalyzing(false);
    }
  };

  const handleKindChange = (k: string) => {
    setKind(k);
    // Remap the auto-detected fields wherever the new kind changes meaning.
    setRestored((prev) => prev.map((f, i) => {
      const col = analysis?.columns[i];
      if (!col) return f;
      const base = f;
      if (k === "outgoing" && (base === "sender" || base === "arrival_date")) {
        return "";
      }
      if (k === "incoming" && base === "recipient") {
        return col.field ?? "";
      }
      return base;
    }));
  };

  const handleFieldChange = (i: number, value: string) => {
    setRestored((prev) => {
      const next = [...prev];
      next[i] = value;
      return next;
    });
  };

  const canImport =
    analysis &&
    analysis.rows.length > 0 &&
    restored.some((f) => f === "registration_number") &&
    !importing;

  const handleImport = async () => {
    if (!analysis || !canImport) return;
    setImporting(true);
    setError(null);
    setResult(null);
    try {
      const columns: ExcelColumn[] = analysis.columns.map((c, i) => ({
        header: c.header,
        group: c.group,
        field: restored[i] || null,
      }));
      const res = await excelCommands.import({
        file_name: analysis.file_name,
        kind,
        columns,
        rows: analysis.rows,
      });
      setResult(res);
      if (res.imported > 0) {
        if (kind === "incoming") {
          qc.invalidateQueries({ queryKey: ["incoming"] });
        } else {
          qc.invalidateQueries({ queryKey: ["outgoing"] });
        }
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setImporting(false);
    }
  };

  const reset = () => {
    setAnalysis(null);
    setResult(null);
    setError(null);
    setRestored([]);
    setKind("incoming");
  };

  const previewCols = analysis ? analysis.columns.slice(0, MAX_PREVIEW) : [];

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div className="bg-white rounded-xl shadow-xl w-[860px] max-h-[92vh] flex flex-col animate-scale-in">
        <div className="flex items-center justify-between px-6 py-4 border-b border-gray-200">
          <div className="flex items-center gap-3">
            <div className="w-9 h-9 rounded-lg bg-emerald-100 flex items-center justify-center">
              <FileSpreadsheet className="w-5 h-5 text-emerald-600" />
            </div>
            <h2 className="text-lg font-bold text-slate-900">استيراد من Excel</h2>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 text-slate-400 hover:text-slate-700 hover:bg-slate-100 rounded-lg transition"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="flex-1 overflow-auto p-6 space-y-6">
          {error && (
            <div className="flex items-start gap-2 text-sm text-red-700 bg-red-50 border border-red-100 rounded-lg px-4 py-3">
              <AlertTriangle className="w-4 h-4 mt-0.5 shrink-0" />
              <span>{error}</span>
            </div>
          )}

          {!analysis && !analyzing && (
            <div className="text-center py-8">
              <p className="text-sm text-slate-500 mb-4">
                اختر ملف Excel (.xlsx أو .xls) وسيتم تحليله تلقائياً واستيراد السجلات إلى القاعدة.
              </p>
              <button
                onClick={handlePick}
                className="inline-flex items-center gap-2 px-5 py-2.5 text-sm font-semibold text-white bg-emerald-600 hover:bg-emerald-700 rounded-lg shadow-sm transition"
              >
                <FileSpreadsheet className="w-4 h-4" />
                اختيار ملف Excel
              </button>
            </div>
          )}

          {analyzing && (
            <div className="flex flex-col items-center gap-3 py-12 text-slate-500">
              <Loader2 className="w-8 h-8 animate-spin" />
              <span className="text-sm">جارٍ تحليل الملف...</span>
            </div>
          )}

          {analysis && (
            <>
              <div className="flex flex-wrap items-center gap-4">
                <div className="min-w-0">
                  <p className="text-xs text-slate-400">الملف</p>
                  <p className="text-sm font-semibold text-slate-900 truncate max-w-[280px]">
                    {analysis.file_name}
                  </p>
                  <p className="text-xs text-slate-400 mt-0.5">
                    الورقة: {analysis.sheet_name || "—"}
                  </p>
                </div>

                <div className="min-w-0">
                  <p className="text-xs text-slate-400">نوع البيانات</p>
                  {analysis.kind_confident ? (
                    <div className="flex items-center gap-1.5 text-sm font-semibold text-slate-900">
                      {kind === "outgoing" ? "صادرات" : "واردات"}
                      <CheckCircle2 className="w-4 h-4 text-emerald-500" />
                    </div>
                  ) : (
                    <select
                      value={kind}
                      onChange={(e) => handleKindChange(e.target.value)}
                      className="mt-0.5 px-2 py-1 text-sm border border-gray-300 rounded-md bg-white focus:outline-none focus:ring-2 focus:ring-emerald-500"
                    >
                      <option value="incoming">واردات</option>
                      <option value="outgoing">صادرات</option>
                    </select>
                  )}
                </div>

                <div className="grid grid-cols-4 gap-3 ms-auto text-center">
                  <div className="px-3 py-2 rounded-lg bg-gray-50">
                    <p className="text-lg font-bold text-slate-900">{analysis.total_rows}</p>
                    <p className="text-xs text-slate-400">إجمالي</p>
                  </div>
                  <div className="px-3 py-2 rounded-lg bg-emerald-50">
                    <p className="text-lg font-bold text-emerald-600">{analysis.valid_rows}</p>
                    <p className="text-xs text-slate-400">صالح</p>
                  </div>
                  <div className="px-3 py-2 rounded-lg bg-red-50">
                    <p className="text-lg font-bold text-red-600">{analysis.invalid_rows}</p>
                    <p className="text-xs text-slate-400">غير صالح</p>
                  </div>
                  <div className="px-3 py-2 rounded-lg bg-amber-50">
                    <p className="text-lg font-bold text-amber-600">{analysis.duplicate_rows}</p>
                    <p className="text-xs text-slate-400">مكرر</p>
                  </div>
                </div>
              </div>

              {analysis.sample_issues.length > 0 && (
                <div className="text-sm bg-amber-50 border border-amber-100 rounded-lg px-4 py-3">
                  <p className="font-semibold text-amber-800 mb-1">
                    <AlertTriangle className="w-4 h-4 inline -mt-0.5 me-1" />
                    تنبيهات ({analysis.sample_issues.length})
                  </p>
                  <ul className="space-y-0.5 text-amber-700">
                    {analysis.sample_issues.map((issue, i) => (
                      <li key={i}>الصف {issue.source_row}: {issue.reason}</li>
                    ))}
                  </ul>
                </div>
              )}

              <div>
                <p className="text-sm font-semibold text-slate-700 mb-2">تطابق الأعمدة</p>
                <div className="border border-gray-200 rounded-lg overflow-hidden">
                  <table className="w-full text-sm">
                    <thead className="bg-gray-50 text-slate-500 text-xs">
                      <tr>
                        <th className="text-right font-medium px-3 py-2">العمود في الملف</th>
                        <th className="text-right font-medium px-3 py-2">الحقل المستورد إليه</th>
                      </tr>
                    </thead>
                    <tbody>
                      {analysis.columns.map((col, i) => (
                        <tr key={i} className="border-t border-gray-100">
                          <td className="px-3 py-1.5 text-slate-700">{col.header}</td>
                          <td className="px-3 py-1.5">
                            <select
                              value={restored[i] ?? ""}
                              onChange={(e) => handleFieldChange(i, e.target.value)}
                              className="w-full px-2 py-1 text-sm border border-gray-300 rounded-md bg-white focus:outline-none focus:ring-2 focus:ring-emerald-500"
                            >
                              <option value="">تجاهل</option>
                              {FIELD_LABELS.map((f) => (
                                <option key={f.field} value={f.field}>
                                  {f.label}
                                </option>
                              ))}
                            </select>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>

              {analysis.preview.length > 0 && (
                <div>
                  <p className="text-sm font-semibold text-slate-700 mb-2">
                    معاينة (أول {Math.min(previewCols.length, analysis.preview.length)} صفوف)
                  </p>
                  <div className="border border-gray-200 rounded-lg overflow-auto max-h-64">
                    <table className="w-full text-sm">
                      <thead className="bg-gray-50 text-slate-500 text-xs sticky top-0">
                        <tr>
                          <th className="text-right font-medium px-3 py-2">#</th>
                          {previewCols.map((col, ci) => (
                            <th key={ci} className="text-right font-medium px-3 py-2">
                              {fieldName(col.field) || col.header}
                            </th>
                          ))}
                        </tr>
                      </thead>
                      <tbody>
                        {analysis.preview.map((row, ri) => (
                          <tr key={ri} className="border-t border-gray-100">
                            <td className="px-3 py-1.5 text-slate-400">{row.source_row}</td>
                            {previewCols.map((_, ci) => (
                              <td key={ci} className="px-3 py-1.5 text-slate-700">
                                {row.cells[ci] || ""}
                              </td>
                            ))}
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              )}

              {result && (
                <div className="text-sm bg-emerald-50 border border-emerald-200 rounded-lg px-4 py-3 text-emerald-800">
                  <p className="font-semibold mb-1">
                    <CheckCircle2 className="w-4 h-4 inline -mt-0.5 me-1" />
                    تم الاستيراد
                  </p>
                  <ul className="space-y-0.5">
                    <li>تم إدخال {result.imported} من أصل {result.total} سجل.</li>
                    {result.duplicates > 0 && (
                      <li>تم تخطي {result.duplicates} سجل مكرر.</li>
                    )}
                    {result.errors > 0 && (
                      <li>حدث خطأ في {result.errors} سجل.</li>
                    )}
                  </ul>
                  {result.failures.length > 0 && (
                    <ul className="mt-2 space-y-0.5 text-red-700">
                      {result.failures.slice(0, 10).map((f, i) => (
                        <li key={i}>الصف {f.source_row}: {f.reason}</li>
                      ))}
                    </ul>
                  )}
                  <button
                    onClick={reset}
                    className="mt-3 px-4 py-1.5 text-sm font-semibold text-emerald-700 bg-white border border-emerald-300 rounded-lg hover:bg-emerald-100 transition"
                  >
                    استيراد ملف آخر
                  </button>
                </div>
              )}
            </>
          )}
        </div>

        <div className="flex items-center gap-3 justify-between px-6 py-4 border-t border-gray-200">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm font-medium text-slate-600 bg-gray-100 rounded-lg hover:bg-gray-200 transition"
          >
            إغلاق
          </button>
          {analysis && !result && (
            <button
              onClick={handleImport}
              disabled={!canImport}
              className="px-5 py-2 text-sm font-semibold text-white bg-emerald-600 hover:bg-emerald-700 rounded-lg disabled:opacity-50 transition flex items-center gap-2"
            >
              {importing && <Loader2 className="w-4 h-4 animate-spin" />}
              استيراد البيانات ({analysis.rows.length})
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
