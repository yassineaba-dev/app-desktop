import { X } from "lucide-react";
import { FileText, Calendar, User, BookOpen, Hash } from "lucide-react";
import type { Incoming } from "@/db/types";
import { formatSequentialNumber } from "@/lib/utils";

interface Props {
  item: Incoming;
  onClose: () => void;
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

function Field({ icon: Icon, label, value }: { icon: React.ElementType; label: string; value: string }) {
  return (
    <div className="flex items-start gap-3 py-3">
      <div className="w-8 h-8 rounded-lg bg-gray-100 flex items-center justify-center shrink-0 mt-0.5">
        <Icon className="w-4 h-4 text-slate-500" />
      </div>
      <div className="flex-1 min-w-0">
        <p className="text-xs text-slate-400 mb-0.5">{label}</p>
        <p className="text-sm font-medium text-slate-800 break-words">{value}</p>
      </div>
    </div>
  );
}

export function IncomingReview({ item, onClose }: Props) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div className="bg-white rounded-xl shadow-xl w-[560px] max-h-[90vh] flex flex-col animate-scale-in">
        <div className="flex items-center justify-between px-6 py-4 border-b border-gray-200">
          <div>
            <h2 className="text-lg font-bold text-slate-900">مراجعة الوارد</h2>
            <p className="text-xs text-slate-500">{formatSequentialNumber(item.registration_number, item.is_duplicate)}</p>
          </div>
          <button onClick={onClose} className="p-1.5 text-slate-400 hover:text-slate-700 hover:bg-slate-100 rounded-lg transition">
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="flex-1 overflow-auto p-6">
          <div className="divide-y divide-gray-200">
            <Field icon={Hash} label="الرقم الترتيبي" value={formatSequentialNumber(item.registration_number, item.is_duplicate)} />
            <Field icon={FileText} label="رقمها" value={item.correspondence_number || "—"} />
            <Field icon={Calendar} label="تاريخ الرسالة" value={formatEnDate(item.date)} />
            <Field icon={Calendar} label="تاريخ الوصول" value={formatEnDate(item.arrival_date ?? "")} />
            <Field icon={User} label="اسم و موطن المرسل" value={item.sender} />
            <Field icon={BookOpen} label="الموضوع" value={item.subject} />
          </div>
          <p className="text-xs text-slate-400 mt-4 text-center">
            آخر تحديث: {formatEnDate(item.updated_at)}
          </p>
        </div>

      </div>
    </div>
  );
}
