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
import { cn, formatSequentialNumber } from "@/lib/utils";
import { useDeleteIncoming, useCreateIncoming } from "@/hooks/use-database";
import { useSharedRowHeights } from "@/hooks/use-shared-row-heights";
import type { Incoming, CreateIncomingData, IncomingFileInfo } from "@/db/types";
import { incomingCommands } from "@/db/commands";

interface Props {
  items: Incoming[];
  total: number;
  page: number;
  perPage: number;
  isLoading: boolean;
  search: string;
  onSearchChange: (v: string) => void;
  dateFilter: string;
  onDateFilterChange: (v: string) => void;
  onPageChange: (p: number) => void;
  onEdit: (item: Incoming) => void;
}

const HEADERS = ["الرقم الترتيبي", "تاريخ الرسالة", "رقمها", "تاريخ الوصول", "اسم و موطن المرسل", "الموضوع", "المراسلة"];

const INITIAL_QUICK: CreateIncomingData = {
  registration_number: "",
  correspondence_number: "",
  date: "",
  arrival_date: "",
  subject: "",
  sender: "",
  destination_service: "",
  is_duplicate: false,
};

function formatEnDate(iso: string): string {
  if (!iso) return "—";
  try {
    const d = new Date(iso);
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, "0");
    const day = String(d.getDate()).padStart(2, "0");
    return `${y}/${m}/${day}`;
  } catch {
    return iso.slice(0, 10);
  }
}

export function IncomingTable({
  items,
  total,
  page,
  perPage,
  isLoading,
  search,
  onSearchChange,
  dateFilter,
  onDateFilterChange,
  onPageChange,
  onEdit,
}: Props) {
  const deleteMutation = useDeleteIncoming();
  const createMutation = useCreateIncoming();
  const [deleteId, setDeleteId] = useState<string | null>(null);
  const [quick, setQuick] = useState<CreateIncomingData>({ ...INITIAL_QUICK });
  const [quickFile, setQuickFile] = useState<IncomingFileInfo | null>(null);
  const [pickingFile, setPickingFile] = useState(false);
  const [downloadingId, setDownloadingId] = useState<string | null>(null);
  const [quickError, setQuickError] = useState<string | null>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  const mountedRef = useRef(true);
  const { tbodyRef: dataTbodyRef, heights: sharedHeights } = useSharedRowHeights<Incoming>(items, "incoming");

  useEffect(() => {
    mountedRef.current = true;
    return () => { mountedRef.current = false; };
  }, []);

  const totalPages = Math.max(1, Math.ceil(total / perPage));

  const handleQuickChange = (key: keyof CreateIncomingData, value: string) => {
    setQuick((f) => ({ ...f, [key]: value }));
    if (quickError) setQuickError(null);
  };

  const toggleQuickDuplicate = () => {
    setQuick((f) => ({ ...f, is_duplicate: !f.is_duplicate }));
    if (quickError) setQuickError(null);
  };

  const handleQuickSubmit = () => {
    if (!quick.registration_number.trim()) {
      setQuickError("الرقم الترتيبي مطلوب");
      return;
    }
    if (!quick.date.trim()) {
      setQuickError("تاريخ الرسالة مطلوب");
      return;
    }
    if (!quick.subject.trim()) {
      setQuickError("الموضوع مطلوب");
      return;
    }
    if (!quick.sender.trim()) {
      setQuickError("اسم و موطن المرسل مطلوب");
      return;
    }

    createMutation.mutate(
      {
        registration_number: quick.registration_number.trim(),
        correspondence_number: quick.correspondence_number?.trim() || undefined,
        date: quick.date,
        arrival_date: quick.arrival_date || undefined,
        subject: quick.subject.trim(),
        sender: quick.sender.trim(),
        destination_service: quick.destination_service?.trim() || "",
        is_duplicate: !!quick.is_duplicate,
        file_name: quickFile?.file_name || undefined,
        file_path: quickFile?.file_path || undefined,
      },
      {
        onSuccess: () => {
          setQuick({ ...INITIAL_QUICK });
          setQuickFile(null);
          setQuickError(null);
          if (mountedRef.current) searchRef.current?.focus();
        },
      },
    );
  };

  const handlePickQuickFile = async () => {
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
      setPickingFile(true);
      const info = await incomingCommands.saveFile(selected);
      setQuickFile(info);
      if (quickError) setQuickError(null);
    } catch (e) {
      setQuickError(e instanceof Error ? e.message : String(e));
    } finally {
      setPickingFile(false);
    }
  };

  const handleDownload = async (item: Incoming) => {
    if (downloadingId) return;
    if (!item.file_path) return;
    setDownloadingId(item.id);
    try {
      await incomingCommands.downloadFile(item.id);
    } catch (e) {
      setQuickError(e instanceof Error ? e.message : String(e));
    } finally {
      setDownloadingId(null);
    }
  };

  const handleQuickKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleQuickSubmit();
    }
  };

  const handleDelete = () => {
    if (deleteId) {
      deleteMutation.mutate(deleteId, {
        onSuccess: () => setDeleteId(null),
      });
    }
  };

  const quickInputClass =
    "w-full px-2.5 py-2 text-sm border border-gray-300 rounded bg-white text-slate-900 placeholder:text-slate-400 focus:outline-none focus:ring-1 focus:ring-blue-200 focus:border-blue-300 transition";
  const quickTextAreaClass =
    "w-full px-2.5 py-2 text-sm border border-gray-300 rounded bg-white text-slate-900 placeholder:text-slate-400 focus:outline-none focus:ring-1 focus:ring-blue-200 focus:border-blue-300 transition resize-none min-h-[3.5rem]";

  return (
    <div className="flex flex-col flex-1 overflow-hidden">
      {/* Toolbar */}
      <div className="px-4 sm:px-6 py-4 bg-white border-b border-gray-200 shrink-0">
        <div className="flex flex-wrap items-center gap-3">
          <div className="relative flex-1 min-w-[180px] max-w-sm">
            <Search className="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400 pointer-events-none" />
            <input
              ref={searchRef}
              type="text"
              value={search}
              onChange={(e) => {
                onSearchChange(e.target.value);
                onPageChange(1);
              }}
              placeholder="بحث في الواردات..."
              className="w-full pr-10 pl-4 py-2 text-sm border border-gray-300 rounded-lg bg-white text-slate-900 placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-200 focus:border-blue-300 transition"
            />
          </div>
          <div className="text-sm text-slate-500">
            {total} سجل
          </div>
        </div>
        <div className="flex flex-wrap items-center gap-2 mt-3">
          <span className="text-sm text-slate-500">تصفية بالتاريخ:</span>
          <input
            type="date"
            value={dateFilter}
            onChange={(e) => { onDateFilterChange(e.target.value); onPageChange(1); }}
            className="px-2 py-1.5 text-sm border border-gray-300 rounded-lg bg-white text-slate-900 focus:outline-none focus:ring-2 focus:ring-blue-200 focus:border-blue-300 transition"
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

      {/* Table */}
      <div className="flex-1 overflow-auto">
        {isLoading ? (
          <div className="flex items-center justify-center h-64">
            <Loader2 className="w-6 h-6 text-slate-400 animate-spin" />
          </div>
        ) : (
          <table className="w-full min-w-[900px] text-sm table-fixed" dir="rtl">
            <thead className="bg-gray-50 border-b border-gray-200 sticky top-0 z-10">
              <tr>
                {HEADERS.map((_, i) => (
                  <th key={i} className="px-3 py-2 bg-gray-50 border-l border-gray-200" />
                ))}
                <th className="px-3 py-2 bg-gray-50 border-l border-gray-200" />
              </tr>
              <tr>
                {HEADERS.map((label, i) => (
                  <th
                    key={i}
                    className={`px-3 pt-3 pb-[23.5px] text-center text-xs font-semibold text-slate-600 whitespace-nowrap border-l border-gray-200 ${label === "المراسلة" ? "border-r border-r-gray-200" : ""} ${label === "اسم و موطن المرسل" ? "pl-8" : ""}`}
                  >
                    {label}
                  </th>
                ))}
                <th className="px-3 pt-3 pb-[23.5px] text-center text-xs font-semibold text-slate-600 border-l border-gray-200">
                  الإجراءات
                </th>
              </tr>
            </thead>
            <tbody ref={dataTbodyRef} className="divide-y divide-gray-100 bg-white">
              {/* Quick-add row */}
              <tr className="bg-slate-50 border-b-2 border-slate-200" onKeyDown={handleQuickKeyDown}>
                <td className="px-2 py-1.5 border-l border-gray-200">
                  <div className="flex flex-col items-center gap-1.5">
                    <input
                      type="text"
                      value={quick.registration_number}
                      onChange={(e) => handleQuickChange("registration_number", e.target.value)}
                      className={`${quickInputClass} text-center`}
                      placeholder="الترتيبي"
                    />
                    <label className="inline-flex items-center gap-1.5 text-xs text-slate-600 cursor-pointer select-none">
                      <input
                        type="checkbox"
                        checked={!!quick.is_duplicate}
                        onChange={toggleQuickDuplicate}
                        className="accent-amber-600 w-3.5 h-3.5 cursor-pointer"
                      />
                      مكرر
                    </label>
                  </div>
                </td>
                <td className="px-2 py-1.5 border-l border-gray-200">
                  <input
                    type="date"
                    value={quick.date}
                    onChange={(e) => handleQuickChange("date", e.target.value)}
                    className={quickInputClass}
                  />
                </td>
                <td className="px-2 py-1.5 border-l border-gray-200">
                  <input
                    type="text"
                    value={quick.correspondence_number ?? ""}
                    onChange={(e) => handleQuickChange("correspondence_number", e.target.value)}
                    className={`${quickInputClass} text-center`}
                    placeholder="رقمها"
                  />
                </td>
                <td className="px-2 py-1.5 border-l border-gray-200">
                  <input
                    type="date"
                    value={quick.arrival_date ?? ""}
                    onChange={(e) => handleQuickChange("arrival_date", e.target.value)}
                    className={quickInputClass}
                  />
                </td>
                <td className="px-2 py-1.5 border-l border-gray-200">
                  <textarea
                    rows={3}
                    value={quick.sender}
                    onChange={(e) => handleQuickChange("sender", e.target.value)}
                    className={quickTextAreaClass}
                    placeholder="اسم و موطن المرسل"
                  />
                </td>
                {/* الموضوع */}
                <td className="px-2 py-1.5 border-l border-gray-200">
                  <textarea
                    rows={3}
                    value={quick.subject}
                    onChange={(e) => handleQuickChange("subject", e.target.value)}
                    className={quickTextAreaClass}
                    placeholder="الموضوع"
                  />
                </td>
                {/* المراسلة */}
                <td className="px-2 py-1.5 border-l border-gray-200 border-r border-r-gray-200 text-center">
                  <div className="flex flex-col items-center gap-1">
                    <button
                      type="button"
                      onClick={handlePickQuickFile}
                      disabled={pickingFile}
                      className="p-1.5 text-blue-600 bg-blue-50 hover:bg-blue-100 rounded-md transition-colors disabled:opacity-50"
                      title={quickFile ? `الملف: ${quickFile.file_name}` : "إرفاق ملف (Word/PDF)"}
                    >
                      {pickingFile ? (
                        <Loader2 className="w-4 h-4 animate-spin" />
                      ) : (
                        <Paperclip className="w-4 h-4" />
                      )}
                    </button>
                    {quickFile && (
                      <button
                        type="button"
                        onClick={() => setQuickFile(null)}
                        className="text-[10px] text-slate-500 hover:text-red-600 truncate max-w-[80px]"
                        title={quickFile.file_name}
                      >
                        {quickFile.file_name}
                      </button>
                    )}
                  </div>
                </td>
                <td className="px-2 py-1.5 border-l border-gray-200 text-center">
                  <div className="inline-flex items-center justify-center gap-1">
                    <button
                      onClick={handleQuickSubmit}
                      disabled={createMutation.isPending}
                      className="p-1.5 text-white bg-emerald-600 hover:bg-emerald-700 rounded-md transition-colors disabled:opacity-50"
                      title="حفظ (Enter)"
                    >
                      {createMutation.isPending ? (
                        <Loader2 className="w-4 h-4 animate-spin" />
                      ) : (
                        <Check className="w-4 h-4" />
                      )}
                    </button>
                    <button
                      onClick={() => {
                        setQuick({ ...INITIAL_QUICK });
                        setQuickFile(null);
                        setQuickError(null);
                      }}
                      className="p-1.5 text-slate-400 hover:text-slate-700 hover:bg-slate-200 rounded-md transition-colors"
                      title="مسح"
                    >
                      <X className="w-4 h-4" />
                    </button>
                  </div>
                </td>
              </tr>

              {items.length === 0 && !search && (
                <tr>
                  <td colSpan={8} className="text-center py-16 text-slate-400">
                    <Inbox className="w-12 h-12 mx-auto mb-3 text-slate-300" />
                    <p className="text-sm">لا توجد سجلات واردة — أضف أول سجل أعلاه</p>
                  </td>
                </tr>
              )}
              {items.length === 0 && search && (
                <tr>
                  <td colSpan={8} className="text-center py-16 text-slate-400">
                    <p className="text-sm">لا توجد نتائج مطابقة للبحث</p>
                  </td>
                </tr>
              )}
              {items.map((item, idx) => (
                    <tr key={item.id} data-idx={idx} style={{ height: sharedHeights[idx] }} className="hover:bg-gray-50/80 transition-colors">
                  <td className="px-3 py-3 text-slate-700 break-words border-l border-gray-200 min-h-[4rem]">
                    {formatSequentialNumber(item.registration_number, item.is_duplicate)}
                  </td>
                  <td className="px-3 py-3 text-slate-700 break-words border-l border-gray-200 min-h-[4rem]">{formatEnDate(item.date)}</td>
                  <td className="px-3 py-3 text-slate-700 break-words border-l border-gray-200 min-h-[4rem]">{item.correspondence_number || "—"}</td>
                  <td className="px-3 py-3 text-slate-700 break-words border-l border-gray-200 min-h-[4rem]">{formatEnDate(item.arrival_date ?? "")}</td>
                  <td className="px-3 py-3 text-slate-700 break-words border-l border-gray-200 min-h-[4rem]">{item.sender}</td>
                  <td className="px-3 py-3 text-slate-700 break-words min-h-[4rem]" title={item.subject}>{item.subject}</td>
                  <td className="px-3 py-3 border-l border-gray-200 border-r border-r-gray-200 min-h-[4rem]">
                    {item.file_path ? (
                      <div className="flex items-center justify-center">
                        <button
                          onClick={() => handleDownload(item)}
                          disabled={downloadingId === item.id}
                          className="p-1.5 text-blue-600 hover:text-blue-700 hover:bg-blue-50 rounded-md transition-colors disabled:opacity-50"
                          title={item.file_name || "تنزيل الملف"}
                        >
                          {downloadingId === item.id ? (
                            <Loader2 className="w-4 h-4 animate-spin" />
                          ) : (
                            <FileDown className="w-4 h-4" />
                          )}
                        </button>
                      </div>
                    ) : (
                      <span className="text-slate-300 text-sm block text-center">—</span>
                    )}
                  </td>
                  <td className="px-3 py-3 text-center border-l border-gray-200 min-h-[4rem]">
                    <div className="inline-flex items-center gap-1 justify-center">
                      <button
                        onClick={() => onEdit(item)}
                        className="p-1.5 text-amber-600 hover:bg-amber-50 rounded-md transition-colors"
                        title="تعديل"
                      >
                        <Pencil className="w-4 h-4" />
                      </button>
                      <button
                        onClick={() => setDeleteId(item.id)}
                        className="p-1.5 text-red-600 hover:bg-red-50 rounded-md transition-colors"
                        title="حذف"
                      >
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

      {/* Quick-add error */}
      {quickError && (
        <div className="px-6 py-2 bg-red-50 border-t border-red-200 text-sm text-red-600 shrink-0">
          {quickError}
          <button onClick={() => setQuickError(null)} className="mr-2 text-red-400 hover:text-red-600">
            ✕
          </button>
        </div>
      )}

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="px-6 py-3 bg-white border-t border-gray-200 flex items-center justify-between shrink-0">
          <span className="text-sm text-slate-500">
            صفحة {page} من {totalPages}
          </span>
          <div className="flex items-center gap-1">
            <button
              onClick={() => onPageChange(Math.max(1, page - 1))}
              disabled={page <= 1}
              className="p-2 text-slate-400 hover:text-slate-700 hover:bg-slate-100 rounded-lg disabled:opacity-30 disabled:cursor-not-allowed transition"
            >
              <ChevronRight className="w-4 h-4" />
            </button>
            {Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
              let pageNum: number;
              if (totalPages <= 5) {
                pageNum = i + 1;
              } else if (page <= 3) {
                pageNum = i + 1;
              } else if (page >= totalPages - 2) {
                pageNum = totalPages - 4 + i;
              } else {
                pageNum = page - 2 + i;
              }
              return (
                <button
                  key={pageNum}
                  onClick={() => onPageChange(pageNum)}
                  className={cn(
                    "w-9 h-9 text-sm rounded-lg transition-colors",
                    pageNum === page
                      ? "bg-slate-800 text-white font-medium"
                      : "text-slate-600 hover:bg-slate-100",
                  )}
                >
                  {pageNum}
                </button>
              );
            })}
            <button
              onClick={() => onPageChange(Math.min(totalPages, page + 1))}
              disabled={page >= totalPages}
              className="p-2 text-slate-400 hover:text-slate-700 hover:bg-slate-100 rounded-lg disabled:opacity-30 disabled:cursor-not-allowed transition"
            >
              <ChevronLeft className="w-4 h-4" />
            </button>
          </div>
        </div>
      )}

      {/* Delete Confirmation */}
      {deleteId && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
          <div className="bg-white rounded-xl shadow-xl p-6 w-[360px] animate-scale-in">
            <h3 className="text-lg font-bold text-slate-900 mb-2">تأكيد الحذف</h3>
            <p className="text-sm text-slate-600 mb-6">
              هل أنت متأكد من حذف هذا السجل؟ لا يمكن التراجع عن هذا الإجراء.
            </p>
            <div className="flex items-center gap-3 justify-end">
              <button
                onClick={() => setDeleteId(null)}
                className="px-4 py-2 text-sm font-medium text-slate-600 bg-gray-100 rounded-lg hover:bg-gray-200 transition"
              >
                إلغاء
              </button>
              <button
                onClick={handleDelete}
                disabled={deleteMutation.isPending}
                className="px-4 py-2 text-sm font-medium text-white bg-red-600 rounded-lg hover:bg-red-700 disabled:opacity-50 transition"
              >
                {deleteMutation.isPending ? "جاري الحذف..." : "حذف"}
              </button>
            </div>
          </div>
        </div>
      )}

    </div>
  );
}
