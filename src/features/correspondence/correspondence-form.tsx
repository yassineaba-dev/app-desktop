import { useState, useEffect } from "react";
import { X, Loader2, Send, Inbox, Paperclip } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { useCreateOutgoing, useUpdateOutgoing } from "@/hooks/use-database";
import { outgoingCommands } from "@/db/commands";
import type { Outgoing, CreateOutgoingData, UpdateOutgoingData, OutgoingFileInfo } from "@/db/types";

interface Props {
  editItem?: Outgoing | null;
  onClose: () => void;
}

function toDateString(iso: string): string {
  if (!iso) return "";
  return iso.slice(0, 10);
}

export function CorrespondenceForm({ editItem, onClose }: Props) {
  const createMutation = useCreateOutgoing();
  const updateMutation = useUpdateOutgoing();
  const isEdit = !!editItem;

  const [form, setForm] = useState({
    registration_number: "",
    date: "",
    recipient: "",
    subject: "",
    correspondence_number: "",
    source: "",
    notes: "",
  });

  const [errors, setErrors] = useState<Record<string, string>>({});
  const [file, setFile] = useState<OutgoingFileInfo | null>(null);
  const [fileIn, setFileIn] = useState<OutgoingFileInfo | null>(null);
  const [pickingFile, setPickingFile] = useState<"out" | "in" | null>(null);

  useEffect(() => {
    if (editItem) {
      setForm({
        registration_number: editItem.registration_number,
        date: toDateString(editItem.date),
        recipient: editItem.recipient,
        subject: editItem.subject,
        correspondence_number: editItem.correspondence_number ?? "",
        source: editItem.source ?? "",
        notes: editItem.notes ?? "",
      });
      setFile(
        editItem.file_path ? { file_name: editItem.file_name ?? "", file_path: editItem.file_path } : null,
      );
      setFileIn(
        editItem.file_path_in ? { file_name: editItem.file_name_in ?? "", file_path: editItem.file_path_in } : null,
      );
    }
  }, [editItem]);

  const set = (key: string, value: string) => {
    setForm((f) => ({ ...f, [key]: value }));
    if (errors[key]) {
      setErrors((e) => { const n = { ...e }; delete n[key]; return n; });
    }
  };

  const validate = (): boolean => {
    const e: Record<string, string> = {};
    if (!form.date.trim()) e.date = "التاريخ مطلوب";
    if (!form.recipient.trim()) e.recipient = "المرسل إليه مطلوب";
    if (!form.subject.trim()) e.subject = "الموضوع مطلوب";
    setErrors(e);
    return Object.keys(e).length === 0;
  };

  const handlePickFile = async (which: "out" | "in" = "out") => {
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
        setFileIn(info);
      } else {
        setFile(info);
      }
    } catch {
      if (which === "in") setFileIn(null); else setFile(null);
    } finally {
      setPickingFile(null);
    }
  };

  const handleSubmit = () => {
    if (!validate()) return;

    if (isEdit && editItem) {
      const payload: UpdateOutgoingData = {
        registration_number: form.registration_number.trim() || undefined,
        date: form.date,
        recipient: form.recipient.trim(),
        subject: form.subject.trim(),
        correspondence_number: form.correspondence_number.trim() || undefined,
        source: form.source.trim() || undefined,
        notes: form.notes.trim() || undefined,
        file_name: file?.file_name || undefined,
        file_path: file?.file_path || undefined,
        file_name_in: fileIn?.file_name || undefined,
        file_path_in: fileIn?.file_path || undefined,
      };
      updateMutation.mutate({ id: editItem.id, data: payload }, { onSuccess: onClose });
    } else {
      const payload: CreateOutgoingData = {
        registration_number: form.registration_number.trim() || `corr-${Date.now()}`,
        date: form.date,
        recipient: form.recipient.trim(),
        subject: form.subject.trim(),
        correspondence_number: form.correspondence_number.trim() || undefined,
        source: form.source.trim() || undefined,
        notes: form.notes.trim() || undefined,
        destination_service: "",
        file_name: file?.file_name || undefined,
        file_path: file?.file_path || undefined,
        file_name_in: fileIn?.file_name || undefined,
        file_path_in: fileIn?.file_path || undefined,
      };
      createMutation.mutate(payload, { onSuccess: onClose });
    }
  };

  const isPending = createMutation.isPending || updateMutation.isPending;

  const inputClass = (key: string) =>
    `w-full px-3 py-2 text-sm border rounded-lg bg-white text-slate-900 placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-slate-800 focus:border-transparent transition ${
      errors[key] ? "border-red-400" : "border-gray-300"
    }`;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div className="bg-white rounded-xl shadow-xl w-[700px] max-h-[90vh] flex flex-col animate-scale-in">
        <div className="flex items-center justify-between px-6 py-4 border-b border-gray-200">
          <h2 className="text-lg font-bold text-slate-900">{isEdit ? "تعديل مراسلة" : "إضافة مراسلة"}</h2>
          <button onClick={onClose} className="p-1.5 text-slate-400 hover:text-slate-700 hover:bg-slate-100 rounded-lg transition">
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="flex-1 overflow-auto p-6 space-y-6">
          <div>
            <div className="flex items-center gap-2 mb-3">
              <Send className="w-4 h-4 text-slate-500" />
              <h3 className="text-sm font-bold text-slate-700">الصادرة</h3>
            </div>
            <div className="grid grid-cols-3 gap-4">
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-1.5">
                  التاريخ <span className="text-red-500">*</span>
                </label>
                <input type="date" value={form.date}
                  onChange={(e) => set("date", e.target.value)}
                  className={inputClass("date")} />
                {errors.date && <p className="text-xs text-red-500 mt-1">{errors.date}</p>}
              </div>
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-1.5">
                  المرسل إليه <span className="text-red-500">*</span>
                </label>
                <input type="text" value={form.recipient}
                  onChange={(e) => set("recipient", e.target.value)}
                  className={inputClass("recipient")} placeholder="المرسل إليه" />
                {errors.recipient && <p className="text-xs text-red-500 mt-1">{errors.recipient}</p>}
              </div>
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-1.5">
                  الموضوع <span className="text-red-500">*</span>
                </label>
                <input type="text" value={form.subject}
                  onChange={(e) => set("subject", e.target.value)}
                  className={inputClass("subject")} placeholder="الموضوع" />
                {errors.subject && <p className="text-xs text-red-500 mt-1">{errors.subject}</p>}
              </div>
            </div>
            <div className="flex items-center gap-3 mt-4">
              <label className="text-sm font-medium text-slate-700 whitespace-nowrap">المراسلة:</label>
              <button
                type="button"
                onClick={() => handlePickFile("out")}
                disabled={pickingFile === "out"}
                className="px-4 py-2 text-sm font-medium text-brand-600 bg-brand-50 rounded-lg hover:bg-brand-100 disabled:opacity-50 transition flex items-center gap-2"
              >
                {pickingFile === "out" ? (
                  <Loader2 className="w-4 h-4 animate-spin" />
                ) : (
                  <Paperclip className="w-4 h-4" />
                )}
                {file ? "تغيير الملف" : "اختيار ملف (Word/PDF)"}
              </button>
              {file && (
                <div className="flex items-center gap-2 text-sm text-slate-600">
                  <span className="truncate max-w-[220px]">{file.file_name}</span>
                  <button
                    type="button"
                    onClick={() => setFile(null)}
                    className="text-red-500 hover:text-red-600"
                    title="إزالة الملف"
                  >
                    <X className="w-4 h-4" />
                  </button>
                </div>
              )}
            </div>
          </div>

          <div className="border-t border-gray-100">
            <div className="flex items-center gap-2 mb-3 pt-4">
              <Inbox className="w-4 h-4 text-slate-500" />
              <h3 className="text-sm font-bold text-slate-700">الواردة</h3>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-1.5">التاريخ</label>
                <input type="date" value={form.correspondence_number}
                  onChange={(e) => set("correspondence_number", e.target.value)}
                  className={inputClass("correspondence_number")} />
              </div>
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-1.5">المصدر والجواب</label>
                <input type="text" value={form.source}
                  onChange={(e) => set("source", e.target.value)}
                  className={inputClass("source")} placeholder="المصدر والجواب" />
              </div>
            </div>
            <div className="flex items-center gap-3 mt-4">
              <label className="text-sm font-medium text-slate-700 whitespace-nowrap">المراسلة:</label>
              <button
                type="button"
                onClick={() => handlePickFile("in")}
                disabled={pickingFile === "in"}
                className="px-4 py-2 text-sm font-medium text-brand-600 bg-brand-50 rounded-lg hover:bg-brand-100 disabled:opacity-50 transition flex items-center gap-2"
              >
                {pickingFile === "in" ? (
                  <Loader2 className="w-4 h-4 animate-spin" />
                ) : (
                  <Paperclip className="w-4 h-4" />
                )}
                {fileIn ? "تغيير الملف" : "اختيار ملف (Word/PDF)"}
              </button>
              {fileIn && (
                <div className="flex items-center gap-2 text-sm text-slate-600">
                  <span className="truncate max-w-[220px]">{fileIn.file_name}</span>
                  <button
                    type="button"
                    onClick={() => setFileIn(null)}
                    className="text-red-500 hover:text-red-600"
                    title="إزالة الملف"
                  >
                    <X className="w-4 h-4" />
                  </button>
                </div>
              )}
            </div>
          </div>

          <div className="border-t border-gray-100">
            <h3 className="text-sm font-bold text-slate-700 mb-3 pt-4">النتيجة (نص)</h3>
            <textarea
              value={form.notes}
              onChange={(e) => set("notes", e.target.value)}
              className="w-full px-3 py-2 text-sm border border-gray-300 rounded-lg bg-white text-slate-900 placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-slate-800 focus:border-transparent transition resize-none h-24"
              placeholder="أدخل نتيجة المراسلة أو الإجراء المتخذ..."
            />
          </div>

        </div>

        <div className="flex items-center gap-3 justify-end px-6 py-4 border-t border-gray-200">
          <button type="button" onClick={onClose}
            className="px-4 py-2 text-sm font-medium text-slate-600 bg-gray-100 rounded-lg hover:bg-gray-200 transition">إلغاء</button>
          <button onClick={handleSubmit} disabled={isPending}
            className="px-5 py-2 text-sm font-medium text-white bg-slate-800 rounded-lg hover:bg-slate-700 disabled:opacity-50 transition flex items-center gap-2">
            {isPending && <Loader2 className="w-4 h-4 animate-spin" />}
            {isEdit ? "تحديث" : "حفظ"}
          </button>
        </div>
      </div>
    </div>
  );
}
