import { useState, useEffect } from "react";
import { X, Loader2, Paperclip } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { useUpdateIncoming } from "@/hooks/use-database";
import { incomingCommands } from "@/db/commands";
import type { Incoming, UpdateIncomingData, IncomingFileInfo } from "@/db/types";

interface Props {
  editItem: Incoming;
  onClose: () => void;
}

function toDateString(iso: string): string {
  if (!iso) return "";
  return iso.slice(0, 10);
}

export function IncomingForm({ editItem, onClose }: Props) {
  const updateMutation = useUpdateIncoming();

  const [form, setForm] = useState<{
    registration_number: string;
    correspondence_number: string;
    date: string;
    arrival_date: string;
    subject: string;
    sender: string;
    notes: string;
    is_duplicate: boolean;
  }>({
    registration_number: "",
    correspondence_number: "",
    date: "",
    arrival_date: "",
    subject: "",
    sender: "",
    notes: "",
    is_duplicate: false,
  });

  const [file, setFile] = useState<IncomingFileInfo | null>(null);
  const [pickingFile, setPickingFile] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});

  useEffect(() => {
    setForm({
      registration_number: editItem.registration_number,
      correspondence_number: editItem.correspondence_number ?? "",
      date: toDateString(editItem.date),
      arrival_date: toDateString(editItem.arrival_date ?? ""),
      subject: editItem.subject,
      sender: editItem.sender,
      notes: editItem.notes ?? "",
      is_duplicate: editItem.is_duplicate,
    });
    setFile(
      editItem.file_path ? { file_name: editItem.file_name ?? "", file_path: editItem.file_path } : null,
    );
  }, [editItem]);

  const set = (key: string, value: string) => {
    setForm((f) => ({ ...f, [key]: value }));
    if (errors[key]) {
      setErrors((e) => {
        const n = { ...e };
        delete n[key];
        return n;
      });
    }
  };

  const validate = (): boolean => {
    const e: Record<string, string> = {};
    if (!form.registration_number.trim()) e.registration_number = "حقل مطلوب";
    if (!form.date.trim()) e.date = "حقل مطلوب";
    if (!form.subject.trim()) e.subject = "حقل مطلوب";
    if (!form.sender.trim()) e.sender = "حقل مطلوب";
    setErrors(e);
    return Object.keys(e).length === 0;
  };

  const handlePickFile = async () => {
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
      setFile(info);
    } catch {
      setFile(null);
    } finally {
      setPickingFile(false);
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!validate()) return;

    const payload: UpdateIncomingData = {
      registration_number: form.registration_number.trim(),
      correspondence_number: form.correspondence_number.trim() || undefined,
      date: form.date,
      arrival_date: form.arrival_date || undefined,
      subject: form.subject.trim(),
      sender: form.sender.trim(),
      notes: form.notes.trim() || undefined,
      is_duplicate: form.is_duplicate,
      file_name: file?.file_name || undefined,
      file_path: file?.file_path || undefined,
    };

    updateMutation.mutate(
      { id: editItem.id, data: payload },
      { onSuccess: onClose },
    );
  };

  const inputClass = (key: string) =>
    `w-full px-3 py-2 text-sm border rounded-lg bg-white text-slate-900 placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-slate-800 focus:border-transparent transition ${
      errors[key] ? "border-red-400" : "border-gray-300"
    }`;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div className="bg-white rounded-xl shadow-xl w-[640px] max-h-[90vh] flex flex-col animate-scale-in">
        <div className="flex items-center justify-between px-6 py-4 border-b border-gray-200">
          <h2 className="text-lg font-bold text-slate-900">تعديل وارد</h2>
          <button
            onClick={onClose}
            className="p-1.5 text-slate-400 hover:text-slate-700 hover:bg-slate-100 rounded-lg transition"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="flex-1 overflow-auto p-6">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-slate-700 mb-1.5">
                الرقم الترتيبي <span className="text-red-500">*</span>
              </label>
              <input
                type="text"
                value={form.registration_number}
                onChange={(e) => set("registration_number", e.target.value)}
                className={inputClass("registration_number")}
                placeholder="أدخل الرقم الترتيبي"
              />
              <label className="inline-flex items-center gap-2 mt-2 text-sm text-slate-700 cursor-pointer select-none">
                <input
                  type="checkbox"
                  checked={form.is_duplicate}
                  onChange={() => setForm((f) => ({ ...f, is_duplicate: !f.is_duplicate }))}
                  className="accent-amber-600 w-4 h-4 cursor-pointer"
                />
                مكرر (نسخة ثانية)
              </label>
              {errors.registration_number && (
                <p className="text-xs text-red-500 mt-1">{errors.registration_number}</p>
              )}
            </div>
            <div>
              <label className="block text-sm font-medium text-slate-700 mb-1.5">
                رقمها
              </label>
              <input
                type="text"
                value={form.correspondence_number}
                onChange={(e) => set("correspondence_number", e.target.value)}
                className={inputClass("correspondence_number")}
                placeholder="أدخل رقم الرسالة"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-slate-700 mb-1.5">
                تاريخ الرسالة <span className="text-red-500">*</span>
              </label>
              <input
                type="date"
                value={form.date}
                onChange={(e) => set("date", e.target.value)}
                className={inputClass("date")}
              />
              {errors.date && (
                <p className="text-xs text-red-500 mt-1">{errors.date}</p>
              )}
            </div>
            <div>
              <label className="block text-sm font-medium text-slate-700 mb-1.5">
                تاريخ الوصول
              </label>
              <input
                type="date"
                value={form.arrival_date}
                onChange={(e) => set("arrival_date", e.target.value)}
                className={inputClass("arrival_date")}
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-slate-700 mb-1.5">
                اسم و موطن المرسل <span className="text-red-500">*</span>
              </label>
              <input
                type="text"
                value={form.sender}
                onChange={(e) => set("sender", e.target.value)}
                className={inputClass("sender")}
                placeholder="اسم و موطن المرسل"
              />
              {errors.sender && (
                <p className="text-xs text-red-500 mt-1">{errors.sender}</p>
              )}
            </div>
            <div>
              <label className="block text-sm font-medium text-slate-700 mb-1.5">
                الموضوع <span className="text-red-500">*</span>
              </label>
              <input
                type="text"
                value={form.subject}
                onChange={(e) => set("subject", e.target.value)}
                className={inputClass("subject")}
                placeholder="أدخل الموضوع"
              />
              {errors.subject && (
                <p className="text-xs text-red-500 mt-1">{errors.subject}</p>
              )}
            </div>

            <div className="col-span-2">
              <label className="block text-sm font-medium text-slate-700 mb-1.5">
                ملاحظات
              </label>
              <textarea
                value={form.notes}
                onChange={(e) => set("notes", e.target.value)}
                className={inputClass("notes") + " resize-none h-20"}
                placeholder="أدخل أي ملاحظات إضافية"
              />
            </div>

            <div className="col-span-2">
              <label className="block text-sm font-medium text-slate-700 mb-1.5">
                المراسلة (ملف مرفق)
              </label>
              <div className="flex items-center gap-3">
                <button
                  type="button"
                  onClick={handlePickFile}
                  disabled={pickingFile}
                  className="px-4 py-2 text-sm font-medium text-blue-600 bg-blue-50 rounded-lg hover:bg-blue-100 disabled:opacity-50 transition flex items-center gap-2"
                >
                  {pickingFile ? (
                    <Loader2 className="w-4 h-4 animate-spin" />
                  ) : (
                    <Paperclip className="w-4 h-4" />
                  )}
                  {file ? "تغيير الملف" : "اختيار ملف (Word/PDF)"}
                </button>
                {file && (
                  <div className="flex items-center gap-2 text-sm text-slate-600">
                    <span className="truncate max-w-[240px]">{file.file_name}</span>
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
          </div>
        </form>

        <div className="flex items-center gap-3 justify-end px-6 py-4 border-t border-gray-200">
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-2 text-sm font-medium text-slate-600 bg-gray-100 rounded-lg hover:bg-gray-200 transition"
          >
            إلغاء
          </button>
          <button
            onClick={handleSubmit}
            disabled={updateMutation.isPending}
            className="px-5 py-2 text-sm font-medium text-white bg-slate-800 rounded-lg hover:bg-slate-700 disabled:opacity-50 transition flex items-center gap-2"
          >
            {updateMutation.isPending && <Loader2 className="w-4 h-4 animate-spin" />}
            تحديث
          </button>
        </div>
      </div>
    </div>
  );
}
