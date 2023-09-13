use actix_web::{guard, web};
use lemmy_api::{
  comment::{distinguish::distinguish_comment, like::like_comment, save::save_comment},
  comment_report::{
    create::create_comment_report,
    list::list_comment_reports,
    resolve::resolve_comment_report,
  },
  community::{
    add_mod::add_mod_to_community,
    ban::ban_from_community,
    block::block_community,
    follow::follow_community,
    hide::hide_community,
    transfer::transfer_community,
  },
  local_user::{
    add_admin::add_admin,
    ban_person::ban_from_site,
    block::block_person,
    change_password::change_password,
    change_password_after_reset::change_password_after_reset,
    get_captcha::get_captcha,
    list_banned::list_banned_users,
    login::login,
    notifications::{
      list_mentions::list_mentions,
      list_replies::list_replies,
      mark_all_read::mark_all_notifications_read,
      mark_mention_read::mark_person_mention_as_read,
      mark_reply_read::mark_reply_as_read,
      unread_count::unread_count,
    },
    report_count::report_count,
    reset_password::reset_password,
    save_settings::save_user_settings,
    verify_email::verify_email,
  },
  post::{
    feature::feature_post,
    get_link_metadata::get_link_metadata,
    like::like_post,
    lock::lock_post,
    mark_read::mark_post_as_read,
    save::save_post,
  },
  post_report::{
    create::create_post_report,
    list::list_post_reports,
    resolve::resolve_post_report,
  },
  private_message::mark_read::mark_pm_as_read,
  private_message_report::{
    create::create_pm_report,
    list::list_pm_reports,
    resolve::resolve_pm_report,
  },
  site::{
    federated_instances::get_federated_instances,
    leave_admin::leave_admin,
    mod_log::get_mod_log,
    purge::{
      comment::purge_comment,
      community::purge_community,
      person::purge_person,
      post::purge_post,
    },
    registration_applications::{
      approve::approve_registration_application,
      list::list_registration_applications,
      unread_count::get_unread_registration_application_count,
    },
  },
  sitemap::get_sitemap,
};
use lemmy_api_crud::{
  comment::{
    create::create_comment,
    delete::delete_comment,
    read::get_comment,
    remove::remove_comment,
    update::update_comment,
  },
  community::{
    create::create_community,
    delete::delete_community,
    list::list_communities,
    remove::remove_community,
    update::update_community,
  },
  custom_emoji::{
    create::create_custom_emoji,
    delete::delete_custom_emoji,
    update::update_custom_emoji,
  },
  post::{
    create::create_post,
    delete::delete_post,
    read::get_post,
    remove::remove_post,
    update::update_post,
  },
  private_message::{
    create::create_private_message,
    delete::delete_private_message,
    read::get_private_message,
    update::update_private_message,
  },
  site::{create::create_site, read::get_site, update::update_site},
  user::{create::register, delete::delete_account},
};
use lemmy_apub::api::{
  list_comments::list_comments,
  list_posts::list_posts,
  read_community::get_community,
  read_person::read_person,
  resolve_object::resolve_object,
  search::search,
};
use lemmy_utils::rate_limit::RateLimitCell;

use crate::session_middleware::SessionMiddleware;

pub fn config(cfg: &mut web::ServiceConfig, rate_limit: &RateLimitCell, auth: &SessionMiddleware) {
  cfg.service(
    web::scope("/api/v3")
      // Site
      .service(
        web::scope("/site")
          .wrap(rate_limit.message())
          .route("", web::get().to(get_site).wrap(auth.opt_auth()))
          // Admin Actions
          .route("", web::post().to(create_site).wrap(auth.auth()))
          .route("", web::put().to(update_site).wrap(auth.auth())),
      )
      .service(
        web::resource("/modlog")
          .wrap(rate_limit.message())
          .route(web::get().to(get_mod_log).wrap(auth.opt_auth())),
      )
      .service(
        web::resource("/search")
          .wrap(rate_limit.search())
          .route(web::get().to(search).wrap(auth.opt_auth())),
      )
      .service(
        web::resource("/resolve_object")
          .wrap(rate_limit.message())
          .route(web::get().to(resolve_object).wrap(auth.opt_auth())),
      )
      // Community
      .service(
        web::resource("/community")
          .guard(guard::Post())
          .wrap(rate_limit.register())
          .route(web::post().to(create_community).wrap(auth.auth())),
      )
      .service(
        web::scope("/community")
          .wrap(rate_limit.message())
          .route("", web::get().to(get_community).wrap(auth.opt_auth()))
          .route("", web::put().to(update_community).wrap(auth.auth()))
          .route("/hide", web::put().to(hide_community).wrap(auth.auth()))
          .route("/list", web::get().to(list_communities).wrap(auth.auth()))
          .route("/follow", web::post().to(follow_community).wrap(auth.auth()))
          .route("/block", web::post().to(block_community).wrap(auth.auth()))
          .route("/delete", web::post().to(delete_community).wrap(auth.auth()))
          // Mod Actions
          .route("/remove", web::post().to(remove_community).wrap(auth.auth()))
          .route("/transfer", web::post().to(transfer_community).wrap(auth.auth()))
          .route("/ban_user", web::post().to(ban_from_community).wrap(auth.auth()))
          .route("/mod", web::post().to(add_mod_to_community).wrap(auth.auth())),
      )
      .service(
        web::scope("/federated_instances")
          .wrap(rate_limit.message())
          .route("", web::get().to(get_federated_instances)),
      )
      // Post
      .service(
        // Handle POST to /post separately to add the post() rate limitter
        web::resource("/post")
          .guard(guard::Post())
          .wrap(rate_limit.post())
          .route(web::post().to(create_post).wrap(auth.auth())),
      )
      .service(
        web::scope("/post")
          .wrap(rate_limit.message())
          .route("", web::get().to(get_post).wrap(auth.opt_auth()))
          .route("", web::put().to(update_post).wrap(auth.auth()))
          .route("/delete", web::post().to(delete_post).wrap(auth.auth()))
          .route("/remove", web::post().to(remove_post).wrap(auth.auth()))
          .route("/mark_as_read", web::post().to(mark_post_as_read).wrap(auth.auth()))
          .route("/lock", web::post().to(lock_post).wrap(auth.auth()))
          .route("/feature", web::post().to(feature_post).wrap(auth.auth()))
          .route("/list", web::get().to(list_posts).wrap(auth.opt_auth()))
          .route("/like", web::post().to(like_post).wrap(auth.auth()))
          .route("/save", web::put().to(save_post).wrap(auth.auth()))
          .route("/report", web::post().to(create_post_report).wrap(auth.auth()))
          .route("/report/resolve", web::put().to(resolve_post_report).wrap(auth.auth()))
          .route("/report/list", web::get().to(list_post_reports).wrap(auth.auth()))
          .route("/site_metadata", web::get().to(get_link_metadata)),
      )
      // Comment
      .service(
        // Handle POST to /comment separately to add the comment() rate limitter
        web::resource("/comment")
          .guard(guard::Post())
          .wrap(rate_limit.comment())
          .route(web::post().to(create_comment).wrap(auth.auth())),
      )
      .service(
        web::scope("/comment")
          .wrap(rate_limit.message())
          .route("", web::get().to(get_comment).wrap(auth.opt_auth()))
          .route("", web::put().to(update_comment).wrap(auth.auth()))
          .route("/delete", web::post().to(delete_comment).wrap(auth.auth()))
          .route("/remove", web::post().to(remove_comment).wrap(auth.auth()))
          .route("/mark_as_read", web::post().to(mark_reply_as_read).wrap(auth.auth()))
          .route("/distinguish", web::post().to(distinguish_comment).wrap(auth.auth()))
          .route("/like", web::post().to(like_comment).wrap(auth.auth()))
          .route("/save", web::put().to(save_comment).wrap(auth.auth()))
          .route("/list", web::get().to(list_comments).wrap(auth.opt_auth()))
          .route("/report", web::post().to(create_comment_report).wrap(auth.auth()))
          .route("/report/resolve", web::put().to(resolve_comment_report).wrap(auth.auth()))
          .route("/report/list", web::get().to(list_comment_reports).wrap(auth.auth())),
      )
      // Private Message
      .service(
        web::scope("/private_message")
          .wrap(rate_limit.message())
          .wrap(auth.auth())
          .route("/list", web::get().to(get_private_message))
          .route("", web::post().to(create_private_message))
          .route("", web::put().to(update_private_message))
          .route("/delete", web::post().to(delete_private_message))
          .route("/mark_as_read", web::post().to(mark_pm_as_read))
          .route("/report", web::post().to(create_pm_report))
          .route("/report/resolve", web::put().to(resolve_pm_report))
          .route("/report/list", web::get().to(list_pm_reports)),
      )
      // User
      .service(
        // Account action, I don't like that it's in /user maybe /accounts
        // Handle /user/register separately to add the register() rate limitter
        web::resource("/user/register")
          .guard(guard::Post())
          .wrap(rate_limit.register())
          .route(web::post().to(register)),
      )
      .service(
        // Handle captcha separately
        web::resource("/user/get_captcha")
          .wrap(rate_limit.post())
          .route(web::get().to(get_captcha)),
      )
      // User actions
      .service(
        web::scope("/user")
          .wrap(rate_limit.message())
          .route("", web::get().to(read_person).wrap(auth.opt_auth()))
          .route("/mention", web::get().to(list_mentions).wrap(auth.auth()))
          .route(
            "/mention/mark_as_read",
            web::post().to(mark_person_mention_as_read).wrap(auth.auth()),
          )
          .route("/replies", web::get().to(list_replies).wrap(auth.auth()))
          // Admin action. I don't like that it's in /user
          .route("/ban", web::post().to(ban_from_site).wrap(auth.auth()))
          .route("/banned", web::get().to(list_banned_users).wrap(auth.auth()))
          .route("/block", web::post().to(block_person).wrap(auth.auth()))
          // Account actions. I don't like that they're in /user maybe /accounts
          .route("/login", web::post().to(login))
          .route("/delete_account", web::post().to(delete_account).wrap(auth.auth()))
          .route("/password_reset", web::post().to(reset_password))
          .route(
            "/password_change",
            web::post().to(change_password_after_reset),
          )
          // mark_all_as_read feels off being in this section as well
          .route(
            "/mark_all_as_read",
            web::post().to(mark_all_notifications_read).wrap(auth.auth()),
          )
          .route("/save_user_settings", web::put().to(save_user_settings).wrap(auth.auth()))
          .route("/change_password", web::put().to(change_password).wrap(auth.auth()))
          .route("/report_count", web::get().to(report_count).wrap(auth.auth()))
          .route("/unread_count", web::get().to(unread_count).wrap(auth.auth()))
          .route("/verify_email", web::post().to(verify_email))
          .route("/leave_admin", web::post().to(leave_admin).wrap(auth.auth())),
      )
      // Admin Actions
      .service(
        web::scope("/admin")
          .wrap(rate_limit.message())
          .wrap(auth.auth())
          .route("/add", web::post().to(add_admin))
          .route(
            "/registration_application/count",
            web::get().to(get_unread_registration_application_count),
          )
          .route(
            "/registration_application/list",
            web::get().to(list_registration_applications),
          )
          .route(
            "/registration_application/approve",
            web::put().to(approve_registration_application),
          )
          .service(
            web::scope("/purge")
              .route("/person", web::post().to(purge_person))
              .route("/community", web::post().to(purge_community))
              .route("/post", web::post().to(purge_post))
              .route("/comment", web::post().to(purge_comment)),
          ),
      )
      .service(
        web::scope("/custom_emoji")
          .wrap(rate_limit.message())
          .wrap(auth.auth())
          .route("", web::post().to(create_custom_emoji))
          .route("", web::put().to(update_custom_emoji))
          .route("/delete", web::post().to(delete_custom_emoji)),
      ),
  );
  cfg.service(
    web::scope("/sitemap.xml")
      .wrap(rate_limit.message())
      .route("", web::get().to(get_sitemap)),
  );
}
