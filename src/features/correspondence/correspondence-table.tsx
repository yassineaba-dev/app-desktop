import { useState, useRef, useEffect } from "react";
import {
  Search,
  Pencil,
  Trash2,
  ChevronLeft,
  ChevronRight,
  Loader2,
  Inbox,
  Check,
  X,
  Paperclip,
  FileDown,
} from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { cn } from "@/lib/utils";
import { useDeleteOutgoing, useCreateOutgoing } from "@/hooks/use-database";
import { useSharedRowHeights } from "@/hooks/use-shared-row-heights";
import { outgoingCommands } from "@/db/commands";
import type { Outgoing, CreateOutgoingData, OutgoingFileInfo } from "@/db/types";

interface Props {
  items: Outgoing[];
  total: number;
  page: number;
  perPage: number;
  isLoading: boolean;
  search: string;
  onSearchChange: (v: string) => void;
  dateFilter: string;
  onDateFilterChange: (v: string) => void;
  onPageChange: (p: number) => void;
  onEdit: (item: Outgoing) => void;
}

const INITIAL_QUICK: CreateOutgoingData = {
  registration_number: "",
  correspondence_number: "",
  date: "",
  subject: "",
  recipient: "",
  destination_service: "",
};

function formatEnDate(iso: string): string {
  if (!iso) return "—";
  try {
    const d = new Date(iso);
    return `${d.getFullYear()}/${String(d.getMonth() + 1).padStart(2, "0")}/${String(d.getDate()).padStart(2, "0")}`;
  } catch {
    return iso.slice(0, 10);
  }
}

export function CorrespondenceTable({
  items, total, page, perPage, isLoading, search,
  onSearchChange, dateFilter, onDateFilterChange,
  onPageChange, onEdit,
}: Props) {
  const deleteMutation = useDeleteOutgoing();
  const createMutation = useCreateOutgoing();
  const [deleteId, setDeleteId] = useState<string | null>(null);
  const [quick, setQuick] = useState<CreateOutgoingData>({ ...INITIAL_QUICK });
  const [quickIncDate, setQuickIncDate] = useState("");
  const [quickSource, setQuickSource] = useState("");
  const [quickResult, setQuickResult] = useState("");
  const [quickFile, setQuickFile] = useState<OutgoingFileInfo | null>(null);
  const [quickFileIn, setQuickFileIn] = useState<OutgoingFileInfo | null>(null);
  const [pickingFile, setPickingFile] = useState<"out" | "in" | null>(null);
  const [downloadingKey, setDownloadingKey] = useState<string | null>(null);
  const [quickError, setQuickError] = useState<string | null>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  const mountedRef = useRef(true);
  const { tbodyRef: dataTbodyRef, heights: sharedHeights } = useSharedRowHeights<Outgoing>(items, "outgoing");

  useEffect(() => {
    mountedRef.current = true;
    return () => { mountedRef.current = false; };
  }, []);

  const totalPages = Math.max(1, Math.ceil(total / perPage));

  const handleQuickChange = (key: keyof CreateOutgoingData, value: string) => {
    setQuick((f) => ({ ...f, [key]: value }));
    if (quickError) setQuickError(null);
  };

  const handleQuickSubmit = () => {
    if (!quick.date.trim()) { setQuickError("التاريخ مطلوب"); return; }
    if (!quick.recipient.trim()) { setQuickError("المرسل إليه مطلوب"); return; }
    if (!quick.subject.trim()) { setQuickError("الموضوع مطلوب"); return; }

    createMutation.mutate({
      registration_number: quick.registration_number.trim() || `corr-${Date.now()}`,
      date: quick.date,
      recipient: quick.recipient.trim(),
      subject: quick.subject.trim(),
      correspondence_number: quickIncDate || undefined,
      source: quickSource.trim() || undefined,
      notes: quickResult.trim() || undefined,
      destination_service: "",
      file_name: quickFile?.file_name || undefined,
      file_path: quickFile?.file_path || undefined,
      file_name_in: quickFileIn?.file_name || undefined,
      file_path_in: quickFileIn?.file_path || undefined,
    }, {
      onSuccess: () => {
        setQuick({ ...INITIAL_QUICK });
        setQuickIncDate("");
        setQuickSource("");
        setQuickResult("");
        setQuickFile(null);
        setQuickFileIn(null);
        setQuickError(null);
        if (mountedRef.current) searchRef.current?.focus();
      },
    });
  };

  const handlePickQuickFile = async (which: "out" | "in" = "out") => {
    if (pickingFile) return;
    try {
      const selected = await open({
        multiple: false,
        filters: [
          { name: "مستندات", extensions: ["pdf", "doc", "docx", "txt", "rtf"] },
          { name: "كل الملفات", extensions: ["*"] },
        ],
      });
      if (!selected || typeof selected !== "string") return;
      setPickingFile(which);
      const info = await outgoingCommands.saveFile(selected);
      if (which === "in") {
        setQuickFileIn(info);
      } else {
        setQuickFile(info);
      }
      if (quickError) setQuickError(null);
    } catch (e) {
      setQuickError(e instanceof Error ? e.message : String(e));
    } finally {
      setPickingFile(null);
    }
  };

  const handleDownload = async (item: Outgoing, which: "out" | "in" = "out") => {
    const key = `${item.id}:${which}`;
    if (downloadingKey) return;
    if (which === "in" ? !item.file_path_in : !item.file_path) return;
    setDownloadingKey(key);
    try {
      if (which === "in") {
        await outgoingCommands.downloadFileIn(item.id);
      } else {
        await outgoingCommands.downloadFile(item.id);
      }
    } catch (e) {
      setQuickError(e instanceof Error ? e.message : String(e));
    } finally {
      setDownloadingKey(null);
    }
  };

  const handleQuickKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); handleQuickSubmit(); }
  };

  const handleDelete = () => {
    if (deleteId) {
      deleteMutation.mutate(deleteId, { onSuccess: () => setDeleteId(null) });
    }
  };

  const quickInputClass =
    "w-full px-2.5 py-2 text-sm border border-gray-300 rounded bg-white text-slate-900 placeholder:text-slate-400 focus:outline-none focus:ring-1 focus:ring-brand-200 focus:border-brand-300 transition";
  const quickTextAreaClass =
    "w-full px-2.5 py-2 text-sm border border-gray-300 rounded bg-white text-slate-900 placeholder:text-slate-400 focus:outline-none focus:ring-1 focus:ring-brand-200 focus:border-brand-300 transition resize-none min-h-[3.5rem]";

  return (
    <div className="flex flex-col flex-1 overflow-hidden">
      <div className="px-4 sm:px-6 py-4 bg-white border-b border-gray-200 shrink-0">
        <div className="flex flex-wrap items-center gap-3">
          <div className="relative flex-1 min-w-[180px] max-w-sm">
            <Search className="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400 pointer-events-none" />
            <input
              ref={searchRef}
              type="text"
              value={search}
              onChange={(e) => { onSearchChange(e.target.value); onPageChange(1); }}
              placeholder="بحث في المراسلات والنتائج..."
              className="w-full pr-10 pl-4 py-2 text-sm border border-gray-300 rounded-lg bg-white text-slate-900 placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-brand-200 focus:border-brand-300 transition"
            />
          </div>
          <div className="text-sm text-slate-500">{total} سجل</div>
        </div>
        <div className="flex flex-wrap items-center gap-2 mt-3">
          <span className="text-sm text-slate-500">تصفية بالتاريخ:</span>
          <input
            type="date"
            value={dateFilter}
            onChange={(e) => { onDateFilterChange(e.target.value); onPageChange(1); }}
            className="px-2 py-1.5 text-sm border border-gray-300 rounded-lg bg-white text-slate-900 focus:outline-none focus:ring-2 focus:ring-brand-200 focus:border-brand-300 transition"
          />
          {dateFilter && (
            <button
              onClick={() => { onDateFilterChange(""); onPageChange(1); }}
              className="px-2 py-1.5 text-sm text-slate-500 hover:text-slate-800 hover:bg-slate-100 rounded-lg transition"
            >
              مسح
            </button>
          )}
        </div>
      </div>

      <div className="flex-1 overflow-auto">
        {isLoading ? (
          <div className="flex items-center justify-center h-64">
            <Loader2 className="w-6 h-6 text-slate-400 animate-spin" />
          </div>
        ) : (
          <table className="w-full min-w-[1150px] text-sm table-fixed" dir="rtl">
            <thead className="bg-gray-50 border-b border-gray-200 sticky top-0 z-10">
              <tr>
                <th className="px-3 py-2 text-center text-xs font-semibold text-slate-700 bg-gray-50 border-l border-gray-200 whitespace-nowrap">
                  الرقم الترتيبي
                </th>
                <th colSpan={4} className="px-3 py-2 text-center text-xs font-semibold text-slate-700 border-l border-gray-200 bg-gray-50 whitespace-nowrap">
                  الصادرة
                </th>
                <th colSpan={3} className="px-3 py-2 text-center text-xs font-semibold text-slate-700 border-l border-gray-200 bg-gray-100/60 whitespace-nowrap">
                  الواردة
                </th>
                <th className="px-3 py-2 text-center text-xs font-semibold text-slate-700 border-l border-gray-200 bg-brand-50 whitespace-nowrap">
                  النتيجة
                </th>
                <th className="px-3 py-2 text-center text-xs font-semibold text-slate-600 border-l border-gray-200">
                  الإجراءات
                </th>
              </tr>
              <tr className="border-b border-gray-300">
                <th className="px-3 py-2.5 text-center text-xs font-medium text-slate-500 whitespace-nowrap border-l border-gray-200">الرقم الترتيبي</th>
                <th className="px-3 py-2.5 text-center text-xs font-medium text-slate-500 whitespace-nowrap border-l border-gray-200">التاريخ</th>
                <th className="px-3 py-2.5 text-center text-xs font-medium text-slate-500 whitespace-nowrap border-l border-gray-200">المرسل إليه</th>
                <th className="px-3 py-2.5 text-center text-xs font-medium text-slate-500 whitespace-nowrap border-l border-gray-200">الموضوع</th>
                <th className="px-3 py-2.5 text-center text-xs font-medium text-slate-500 whitespace-nowrap border-l border-gray-200">المراسلة</th>
                <th className="px-3 py-2.5 text-center text-xs font-medium text-slate-500 whitespace-nowrap border-l border-gray-200">التاريخ</th>
                <th className="px-3 py-2.5 text-center text-xs font-medium text-slate-500 whitespace-nowrap border-l border-gray-200">المصدر والجواب</th>
                <th className="px-3 py-2.5 text-center text-xs font-medium text-slate-500 whitespace-nowrap border-l border-gray-200">المراسلة</th>
                <th className="px-3 py-2.5 text-center text-xs font-medium text-slate-500 whitespace-nowrap border-l border-gray-200">النتيجة</th>
                <th className="px-3 py-2.5 text-center text-xs font-medium text-slate-500 whitespace-nowrap"></th>
              </tr>
            </thead>
            <tbody ref={dataTbodyRef} className="divide-y divide-gray-100 bg-white">
              <tr className="bg-slate-50 border-b-2 border-slate-200" onKeyDown={handleQuickKeyDown}>
                <td className="px-2 py-1.5 border-l border-gray-200 text-center">
                  <input type="text" value={quick.registration_number} onChange={(e) => handleQuickChange("registration_number", e.target.value)} className={`${quickInputClass} text-center`} placeholder="الترتيبي" />
                </td>
                <td className="px-2 py-1.5 border-l border-gray-200 text-center">
                  <input type="date" value={quick.date} onChange={(e) => handleQuickChange("date", e.target.value)} className={quickInputClass} />
                </td>
                <td className="px-2 py-1.5 border-l border-gray-200 text-center">
                  <input type="text" value={quick.recipient} onChange={(e) => handleQuickChange("recipient", e.target.value)} className={quickInputClass} placeholder="المرسل إليه" />
                </td>
                <td className="px-2 py-1.5 border-l border-gray-200 text-center">
                  <textarea rows={3} value={quick.subject} onChange={(e) => handleQuickChange("subject", e.target.value)} className={quickTextAreaClass} placeholder="الموضوع" />
                </td>
                <td className="px-2 py-1.5 border-l border-gray-200">
                  <div className="flex flex-col items-center justify-center h-full min-h-[3.5rem]">
                    {quickFile ? (
                      <span className="text-sm text-slate-600 font-medium break-all">{quickFile.file_name}</span>
                    ) : (
                      <button onClick={() => handlePickQuickFile("out")} disabled={pickingFile === "out"}
                        className="inline-flex items-center justify-center gap-1 text-xs text-brand-600 hover:text-brand-800 border border-dashed border-brand-300 rounded-md px-2 py-1.5 transition-colors disabled:opacity-50"
                        title="إرفاق ملف">
                        {pickingFile === "out" ? <Loader2 className="w-4 h-4 animate-spin" /> : <Paperclip className="w-4 h-4" />}
                        <span>إرفاق</span>
                      </button>
                    )}
                  </div>
                </td>
                <td className="px-2 py-1.5 border-l border-gray-200 text-center">
                  <input type="date" value={quickIncDate} onChange={(e) => { setQuickIncDate(e.target.value); if (quickError) setQuickError(null); }} className={quickInputClass} />
                </td>
                <td className="px-2 py-1.5 border-l border-gray-200 text-center">
                  <textarea rows={3} value={quickSource} onChange={(e) => { setQuickSource(e.target.value); if (quickError) setQuickError(null); }} className={quickTextAreaClass} placeholder="المصدر والجواب" />
                </td>
                <td className="px-2 py-1.5 border-l border-gray-200">
                  <div className="flex flex-col items-center justify-center h-full min-h-[3.5rem]">
                    {quickFileIn ? (
                      <span className="text-sm text-slate-600 font-medium break-all">{quickFileIn.file_name}</span>
                    ) : (
                      <button onClick={() => handlePickQuickFile("in")} disabled={pickingFile === "in"}
                        className="inline-flex items-center justify-center gap-1 text-xs text-brand-600 hover:text-brand-800 border border-dashed border-brand-300 rounded-md px-2 py-1.5 transition-colors disabled:opacity-50"
                        title="إرفاق ملف">
                        {pickingFile === "in" ? <Loader2 className="w-4 h-4 animate-spin" /> : <Paperclip className="w-4 h-4" />}
                        <span>إرفاق</span>
                      </button>
                    )}
                  </div>
                </td>
                <td className="px-2 py-1.5 border-l border-gray-200 text-center">
                  <textarea rows={3} value={quickResult} onChange={(e) => { setQuickResult(e.target.value); if (quickError) setQuickError(null); }} className={quickTextAreaClass} placeholder="النتيجة" />
                </td>
                <td className="px-2 py-1.5 border-l border-gray-200 text-center">
                  <div className="flex flex-wrap items-center gap-1 justify-center">
                    <button onClick={handleQuickSubmit} disabled={createMutation.isPending}
                      className="p-1.5 text-white bg-emerald-600 hover:bg-emerald-700 rounded-md transition-colors disabled:opacity-50" title="حفظ (Enter)">
                      {createMutation.isPending ? <Loader2 className="w-4 h-4 animate-spin" /> : <Check className="w-4 h-4" />}
                    </button>
                    <button onClick={() => { setQuick({ ...INITIAL_QUICK }); setQuickIncDate(""); setQuickSource(""); setQuickResult(""); setQuickFile(null); setQuickError(null); }}
                      className="p-1.5 text-slate-400 hover:text-slate-700 hover:bg-slate-200 rounded-md transition-colors" title="مسح">
                      <X className="w-4 h-4" />
                    </button>
                  </div>
                </td>
              </tr>

              {items.length === 0 && (
                <tr>
                  <td colSpan={10} className="text-center py-16 text-slate-400">
                    <Inbox className="w-12 h-12 mx-auto mb-3 text-slate-300" />
                    <p className="text-sm">{search ? "لا توجد نتائج مطابقة للبحث" : "لا توجد سجلات مراسلات — أضف أول سجل أعلاه"}</p>
                  </td>
                </tr>
              )}
              {items.map((item, idx) => (
                <tr key={item.id} data-idx={idx} style={{ height: sharedHeights[idx] }} className="hover:bg-gray-50/80 transition-colors">
                  <td className="px-3 py-3 text-slate-700 break-words border-l border-gray-200 min-h-[3.5rem]">{item.registration_number}</td>
                  <td className="px-3 py-3 text-slate-700 break-words border-l border-gray-200 min-h-[3.5rem]">{formatEnDate(item.date)}</td>
                  <td className="px-3 py-3 text-slate-700 break-words border-l border-gray-200 min-h-[3.5rem]">{item.recipient}</td>
                  <td className="px-3 py-3 text-slate-700 break-words border-l border-gray-200 min-h-[3.5rem]">{item.subject}</td>
                  <td className="px-3 py-3 text-center border-l border-gray-200 min-h-[3.5rem]">
                    {item.file_path ? (
                      <button onClick={() => handleDownload(item, "out")} disabled={downloadingKey === `${item.id}:out`}
                        className="p-1.5 text-brand-600 hover:bg-brand-50 rounded-md transition-colors inline-flex items-center justify-center" title={item.file_name || "تنزيل الملف"}>
                        {downloadingKey === `${item.id}:out` ? <Loader2 className="w-4 h-4 animate-spin" /> : <FileDown className="w-4 h-4" />}
                      </button>
                    ) : "—"}
                  </td>
                  <td className="px-3 py-3 text-slate-700 break-words border-l border-gray-200 min-h-[3.5rem]">{item.correspondence_number ? formatEnDate(item.correspondence_number) : "—"}</td>
                  <td className="px-3 py-3 text-slate-700 break-words border-l border-gray-200 min-h-[3.5rem]">{item.source || "—"}</td>
                  <td className="px-3 py-3 text-center border-l border-gray-200 min-h-[3.5rem]">
                    {item.file_path_in ? (
                      <button onClick={() => handleDownload(item, "in")} disabled={downloadingKey === `${item.id}:in`}
                        className="p-1.5 text-brand-600 hover:bg-brand-50 rounded-md transition-colors inline-flex items-center justify-center" title={item.file_name_in || "تنزيل الملف"}>
                        {downloadingKey === `${item.id}:in` ? <Loader2 className="w-4 h-4 animate-spin" /> : <FileDown className="w-4 h-4" />}
                      </button>
                    ) : "—"}
                  </td>
                  <td className="px-3 py-3 text-slate-700 border-l border-gray-200 whitespace-normal min-h-[3.5rem]">{item.notes || "—"}</td>
                  <td className="px-3 py-3 text-center border-l border-gray-200 min-h-[3.5rem]">
                    <div className="flex items-center gap-1 justify-center">
                      <button onClick={() => onEdit(item)} className="p-1.5 text-amber-600 hover:bg-amber-50 rounded-md transition-colors" title="تعديل">
                        <Pencil className="w-4 h-4" />
                      </button>
                      <button onClick={() => setDeleteId(item.id)} className="p-1.5 text-red-600 hover:bg-red-50 rounded-md transition-colors" title="حذف">
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {quickError && (
        <div className="px-6 py-2 bg-red-50 border-t border-red-200 text-sm text-red-600 shrink-0">
          {quickError}
          <button onClick={() => setQuickError(null)} className="mr-2 text-red-400 hover:text-red-600">✕</button>
        </div>
      )}

      {totalPages > 1 && (
        <div className="px-6 py-3 bg-white border-t border-gray-200 flex items-center justify-between shrink-0">
          <span className="text-sm text-slate-500">صفحة {page} من {totalPages}</span>
          <div className="flex items-center gap-1">
            <button onClick={() => onPageChange(Math.max(1, page - 1))} disabled={page <= 1}
              className="p-2 text-slate-400 hover:text-slate-700 hover:bg-slate-100 rounded-lg disabled:opacity-30 disabled:cursor-not-allowed transition">
              <ChevronRight className="w-4 h-4" />
            </button>
            {Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
              let pageNum: number;
              if (totalPages <= 5) pageNum = i + 1;
              else if (page <= 3) pageNum = i + 1;
              else if (page >= totalPages - 2) pageNum = totalPages - 4 + i;
              else pageNum = page - 2 + i;
              return (
                <button key={pageNum} onClick={() => onPageChange(pageNum)}
                  className={cn("w-9 h-9 text-sm rounded-lg transition-colors",
                    pageNum === page ? "bg-slate-800 text-white font-medium" : "text-slate-600 hover:bg-slate-100")}>
                  {pageNum}
                </button>
              );
            })}
            <button onClick={() => onPageChange(Math.min(totalPages, page + 1))} disabled={page >= totalPages}
              className="p-2 text-slate-400 hover:text-slate-700 hover:bg-slate-100 rounded-lg disabled:opacity-30 disabled:cursor-not-allowed transition">
              <ChevronLeft className="w-4 h-4" />
            </button>
          </div>
        </div>
      )}

      {deleteId && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
          <div className="bg-white rounded-xl shadow-xl p-6 w-[360px] animate-scale-in">
            <h3 className="text-lg font-bold text-slate-900 mb-2">تأكيد الحذف</h3>
            <p className="text-sm text-slate-600 mb-6">هل أنت متأكد من حذف هذا السجل؟ لا يمكن التراجع عن هذا الإجراء.</p>
            <div className="flex items-center gap-3 justify-end">
              <button onClick={() => setDeleteId(null)}
                className="px-4 py-2 text-sm font-medium text-slate-600 bg-gray-100 rounded-lg hover:bg-gray-200 transition">إلغاء</button>
              <button onClick={handleDelete} disabled={deleteMutation.isPending}
                className="px-4 py-2 text-sm font-medium text-white bg-red-600 rounded-lg hover:bg-red-700 disabled:opacity-50 transition">
                {deleteMutation.isPending ? "جاري الحذف..." : "حذف"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
